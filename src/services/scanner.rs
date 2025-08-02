// 区块链扫描器服务
//
// 负责扫描 Tron 区块链，提取交易数据并存储到数据库

use crate::core::{config::Config, database::Database, models::*, tron_client::TronClient};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};

/// 扫描器状态
#[derive(Debug, Clone)]
pub struct ScannerState {
    pub current_block: u64,
    pub latest_block: u64,
    pub is_running: bool,
    pub scan_speed: f64, // 块/分钟
    pub total_transactions: u64,
    pub error_count: u64,
    pub last_error: Option<String>,
}

impl Default for ScannerState {
    fn default() -> Self {
        Self {
            current_block: 0,
            latest_block: 0,
            is_running: false,
            scan_speed: 0.0,
            total_transactions: 0,
            error_count: 0,
            last_error: None,
        }
    }
}

/// 区块链扫描器
#[derive(Clone)]
pub struct Scanner {
    config: Config,
    db: Database,
    tron_client: TronClient,
    state: Arc<RwLock<ScannerState>>,
    notification_sender: Option<mpsc::UnboundedSender<TransactionEvent>>,
}

/// 扫描器服务 (Scanner的别名，为了兼容性)
pub type ScannerService = Scanner;

/// 交易事件，用于通知其他服务
#[derive(Debug, Clone)]
pub struct TransactionEvent {
    pub transaction: Transaction,
    pub event_type: crate::core::models::NotificationEventType,
}

impl Scanner {
    /// 创建新的扫描器实例
    pub fn new(config: Config, db: Database) -> Result<Self> {
        let tron_client = TronClient::new(config.tron.clone())?;

        Ok(Self {
            config,
            db,
            tron_client,
            state: Arc::new(RwLock::new(ScannerState::default())),
            notification_sender: None,
        })
    }

    /// 设置通知发送器
    pub fn set_notification_sender(&mut self, sender: mpsc::UnboundedSender<TransactionEvent>) {
        self.notification_sender = Some(sender);
    }

    /// 获取扫描器状态
    pub async fn get_state(&self) -> ScannerState {
        self.state.read().await.clone()
    }

    /// 扫描器健康检查
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Performing scanner health check");
        let state = self.state.read().await;
        // 简单检查：如果没有错误且正在运行或者已停止但没有错误，认为是健康的
        Ok(state.last_error.is_none())
    }

    /// 启动扫描器
    pub async fn start(&self) -> Result<()> {
        info!("Starting blockchain scanner...");

        // 设置运行状态
        {
            let mut state = self.state.write().await;
            state.is_running = true;
        }

        // 获取起始区块号
        let start_block = self.get_start_block().await?;
        {
            let mut state = self.state.write().await;
            state.current_block = start_block;
        }

        info!("Scanner started from block {}", start_block);

        // 启动扫描循环
        self.scan_loop().await
    }

    /// 停止扫描器
    pub async fn stop(&self) {
        info!("Stopping blockchain scanner...");
        let mut state = self.state.write().await;
        state.is_running = false;
    }

    /// 获取起始区块号
    async fn get_start_block(&self) -> Result<u64> {
        // 首先尝试从数据库获取最后处理的区块
        match self.db.get_last_processed_block().await {
            Ok(Some(last_block)) => {
                info!("Resuming from last processed block: {}", last_block);
                Ok(last_block + 1)
            }
            Ok(None) => {
                // 从system_config获取起始区块
                let start_block = match self.db.get_system_config("start_block").await {
                    Ok(Some(value)) => {
                        match value.as_str() {
                            Some(s) => s.parse::<u64>().unwrap_or(62800000u64),
                            None => value.as_u64().unwrap_or(62800000u64),
                        }
                    }
                    Ok(None) => {
                        // 如果没有配置，使用默认值并保存
                        let default_start_block = 62800000u64;
                        let _ = self.db.save_system_config("start_block", &serde_json::json!(default_start_block.to_string())).await;
                        default_start_block
                    }
                    Err(e) => {
                        warn!("Failed to get start_block from system_config: {}", e);
                        62800000u64
                    }
                };
                info!("Starting from configured block: {}", start_block);
                Ok(start_block)
            }
            Err(e) => {
                warn!("Failed to get last processed block from database: {}", e);
                // 从system_config获取起始区块作为后备
                match self.db.get_system_config("start_block").await {
                    Ok(Some(value)) => {
                        let start_block = match value.as_str() {
                            Some(s) => s.parse::<u64>().unwrap_or(62800000u64),
                            None => value.as_u64().unwrap_or(62800000u64),
                        };
                        Ok(start_block)
                    }
                    _ => Ok(62800000u64)
                }
            }
        }
    }

    /// 主扫描循环
    async fn scan_loop(&self) -> Result<()> {
        let mut scan_interval = interval(Duration::from_secs(self.config.tron.scan_interval));
        let mut speed_calculation_interval = interval(Duration::from_secs(60)); // 每分钟计算一次速度
        let mut last_block_count = 0u64;
        let mut last_speed_check = std::time::Instant::now();

        loop {
            tokio::select! {
                _ = scan_interval.tick() => {
                    if !self.state.read().await.is_running {
                        break;
                    }

                    if let Err(e) = self.scan_next_blocks().await {
                        error!("Scan error: {}", e);
                        self.update_error_state(e.to_string()).await;

                        // 错误后等待一段时间再重试
                        sleep(Duration::from_secs(10)).await;
                    }
                }
                _ = speed_calculation_interval.tick() => {
                    self.calculate_scan_speed(&mut last_block_count, &mut last_speed_check).await;
                }
            }
        }

        info!("Scanner stopped");
        Ok(())
    }

    /// 扫描下一批区块
    async fn scan_next_blocks(&self) -> Result<()> {
        // 获取最新区块号
        let latest_block = self.tron_client.get_latest_block_number().await?;

        // 更新最新区块号
        {
            let mut state = self.state.write().await;
            state.latest_block = latest_block;
        }

        let current_block = self.state.read().await.current_block;

        // 检查是否需要扫描
        if current_block >= latest_block {
            debug!(
                "No new blocks to scan. Current: {}, Latest: {}",
                current_block, latest_block
            );
            return Ok(());
        }

        // 计算要扫描的区块范围
        let batch_size = self.config.tron.batch_size as u64;
        let end_block = std::cmp::min(current_block + batch_size - 1, latest_block);

        // info!("Scanning blocks {} to {}", current_block, end_block);

        // 扫描区块范围
        for block_number in current_block..=end_block {
            if !self.state.read().await.is_running {
                break;
            }

            match self.scan_block(block_number).await {
                Ok(transaction_count) => {
                    debug!(
                        "Scanned block {} with {} transactions",
                        block_number, transaction_count
                    );

                    // 更新状态
                    {
                        let mut state = self.state.write().await;
                        state.current_block = block_number + 1;
                        state.total_transactions += transaction_count;
                    }

                    // 保存进度到数据库
                    if let Err(e) = self.db.save_scan_progress(block_number).await {
                        warn!("Failed to save scan progress: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to scan block {}: {}", block_number, e);
                    self.update_error_state(e.to_string()).await;
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// 扫描单个区块
    async fn scan_block(&self, block_number: u64) -> Result<u64> {
        // debug!("Scanning block {}", block_number);

        // 获取区块数据
        let block_data = self.tron_client.get_block_by_number(block_number).await?;

        // 处理区块中的交易
        let mut processed_transactions = 0u64;

        for transaction in &block_data.transactions {
            match self.process_transaction(transaction).await {
                Ok(_) => {
                    processed_transactions += 1;

                    // 发送交易事件通知
                    if let Some(sender) = &self.notification_sender {
                        let event = TransactionEvent {
                            transaction: transaction.clone(),
                            event_type: crate::core::models::NotificationEventType::Transaction,
                        };

                        if let Err(e) = sender.send(event) {
                            warn!("Failed to send transaction event: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to process transaction {}: {}", transaction.hash, e);
                }
            }
        }

        // 保存区块信息
        if let Err(e) = self.db.save_block(&block_data).await {
            warn!("Failed to save block {}: {}", block_number, e);
        }

        // info!(
        //     "Processed block {} with {} transactions",
        //     block_number, processed_transactions
        // );
        Ok(processed_transactions)
    }

    /// 处理单个交易
    async fn process_transaction(&self, transaction: &Transaction) -> Result<()> {
        debug!("Processing transaction: {}", transaction.hash);

        // 保存交易到数据库
        self.db.save_transaction(transaction).await?;

        // 发送普通交易通知
        if let Some(sender) = &self.notification_sender {
            let event = TransactionEvent {
                transaction: transaction.clone(),
                event_type: crate::core::models::NotificationEventType::Transaction,
            };

            if let Err(e) = sender.send(event) {
                warn!("Failed to send transaction event: {}", e);
            }
        }

        Ok(())
    }

    /// 计算扫描速度
    async fn calculate_scan_speed(
        &self,
        last_block_count: &mut u64,
        last_check: &mut std::time::Instant,
    ) {
        let current_state = self.state.read().await;
        let current_block_count = current_state.current_block;
        let now = std::time::Instant::now();

        let time_elapsed = now.duration_since(*last_check).as_secs_f64() / 60.0; // 转换为分钟

        if time_elapsed > 0.0 && *last_block_count > 0 {
            let blocks_processed = current_block_count.saturating_sub(*last_block_count);
            let speed = blocks_processed as f64 / time_elapsed;

            drop(current_state); // 释放读锁

            let mut state = self.state.write().await;
            state.scan_speed = speed;

            debug!("Scan speed: {:.2} blocks/minute", speed);
        }

        *last_block_count = current_block_count;
        *last_check = now;
    }

    /// 更新错误状态
    async fn update_error_state(&self, error_message: String) {
        let mut state = self.state.write().await;
        state.error_count += 1;
        state.last_error = Some(error_message);
    }

    /// 重置扫描器状态
    pub async fn reset(&self, start_block: Option<u64>) -> Result<()> {
        info!("Resetting scanner...");

        // 停止扫描
        self.stop().await;

        // 重置状态
        {
            let mut state = self.state.write().await;
            *state = ScannerState::default();
            if let Some(block) = start_block {
                state.current_block = block;
            }
        }

        // 清除数据库中的扫描进度（可选）
        if let Some(block) = start_block {
            if let Err(e) = self.db.save_scan_progress(block).await {
                warn!("Failed to reset scan progress: {}", e);
            }
        }

        info!("Scanner reset completed");
        Ok(())
    }

    /// 获取扫描统计信息（为Admin API使用）
    pub async fn get_statistics(&self) -> Result<crate::api::handlers::admin::ScannerStats> {
        let state = self.state.read().await;

        Ok(crate::api::handlers::admin::ScannerStats {
            current_block: state.current_block,
            latest_block: state.latest_block,
            blocks_behind: state.latest_block.saturating_sub(state.current_block),
            scanning_speed: state.scan_speed,
            transactions_processed_today: state.total_transactions,
            errors_today: state.error_count,
            last_scan_time: chrono::Utc::now(),
            scan_status: if state.is_running {
                "running".to_string()
            } else {
                "stopped".to_string()
            },
        })
    }

    /// 获取扫描统计信息（原始版本）
    pub async fn get_scanner_statistics(&self) -> Result<ScannerStatistics> {
        let state = self.state.read().await;

        Ok(ScannerStatistics {
            current_block: state.current_block,
            latest_block: state.latest_block,
            blocks_behind: state.latest_block.saturating_sub(state.current_block),
            scan_speed: state.scan_speed,
            total_transactions: state.total_transactions,
            error_count: state.error_count,
            is_running: state.is_running,
            uptime_seconds: 0, // TODO: 实现运行时间计算
        })
    }

    /// 手动扫描指定区块
    pub async fn scan_specific_block(&self, block_number: u64) -> Result<u64> {
        info!("Manually scanning block {}", block_number);
        self.scan_block(block_number).await
    }

    /// 获取节点健康状态
    pub async fn get_node_health(&self) -> Result<bool> {
        self.tron_client.health_check().await
    }

    /// 重启扫描器
    pub async fn restart(&self) -> Result<()> {
        info!("Restarting scanner...");
        self.stop().await;
        self.start().await
    }

    /// 手动扫描区块（返回扫描结果）
    pub async fn scan_block_admin(&self, block_number: u64) -> Result<ScanBlockResult> {
        let start_time = std::time::Instant::now();
        let transactions_count = self.scan_specific_block(block_number).await?;
        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(ScanBlockResult {
            transactions_count,
            processing_time_ms,
        })
    }
}

/// 扫描器统计信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScannerStatistics {
    pub current_block: u64,
    pub latest_block: u64,
    pub blocks_behind: u64,
    pub scan_speed: f64,
    pub total_transactions: u64,
    pub error_count: u64,
    pub is_running: bool,
    pub uptime_seconds: u64,
}

/// 扫描区块结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanBlockResult {
    pub transactions_count: u64,
    pub processing_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_creation() {
        // Test scanner state initialization without database
        let state = ScannerState {
            current_block: 0,
            latest_block: 0,
            is_running: false,
            scan_speed: 0.0,
            total_transactions: 0,
            error_count: 0,
            last_error: None,
        };

        assert!(!state.is_running);
        assert_eq!(state.current_block, 0);
        assert_eq!(state.total_transactions, 0);
        assert_eq!(state.error_count, 0);
    }
}
