// 区块链扫描器服务
// 
// 负责扫描 Tron 区块链，提取交易数据并存储到数据库

use crate::core::{config::Config, database::Database, models::*, tron_client::TronClient};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{sleep, interval};
use tracing::{info, warn, error, debug};

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
pub struct Scanner {
    config: Config,
    db: Database,
    tron_client: TronClient,
    state: Arc<RwLock<ScannerState>>,
    notification_sender: Option<mpsc::UnboundedSender<TransactionEvent>>,
}

/// 交易事件，用于通知其他服务
#[derive(Debug, Clone)]
pub struct TransactionEvent {
    pub transaction: Transaction,
    pub event_type: String,
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

    /// 启动扫描器
    pub async fn start(&mut self) -> Result<()> {
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
    async fn get_start_block(&mut self) -> Result<u64> {
        // 首先尝试从数据库获取最后处理的区块
        match self.db.get_last_processed_block().await {
            Ok(Some(last_block)) => {
                info!("Resuming from last processed block: {}", last_block);
                Ok(last_block + 1)
            }
            Ok(None) => {
                // 数据库中没有记录，使用配置的起始区块
                let start_block = self.config.blockchain.start_block;
                info!("Starting from configured block: {}", start_block);
                Ok(start_block)
            }
            Err(e) => {
                warn!("Failed to get last processed block from database: {}", e);
                // 使用配置的起始区块作为后备
                Ok(self.config.blockchain.start_block)
            }
        }
    }

    /// 主扫描循环
    async fn scan_loop(&mut self) -> Result<()> {
        let mut scan_interval = interval(Duration::from_secs(self.config.blockchain.scan_interval));
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
    async fn scan_next_blocks(&mut self) -> Result<()> {
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
            debug!("No new blocks to scan. Current: {}, Latest: {}", current_block, latest_block);
            return Ok(());
        }

        // 计算要扫描的区块范围
        let batch_size = self.config.blockchain.batch_size;
        let end_block = std::cmp::min(current_block + batch_size - 1, latest_block);

        info!("Scanning blocks {} to {}", current_block, end_block);

        // 扫描区块范围
        for block_number in current_block..=end_block {
            if !self.state.read().await.is_running {
                break;
            }

            match self.scan_block(block_number).await {
                Ok(transaction_count) => {
                    debug!("Scanned block {} with {} transactions", block_number, transaction_count);
                    
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
    async fn scan_block(&mut self, block_number: u64) -> Result<u64> {
        debug!("Scanning block {}", block_number);

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
                            event_type: "new_transaction".to_string(),
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

        info!("Processed block {} with {} transactions", block_number, processed_transactions);
        Ok(processed_transactions)
    }

    /// 处理单个交易
    async fn process_transaction(&self, transaction: &Transaction) -> Result<()> {
        debug!("Processing transaction: {}", transaction.hash);

        // 保存交易到数据库
        self.db.save_transaction(transaction).await?;

        // 检查是否是大额转账
        if self.is_large_transfer(transaction) {
            info!("Large transfer detected: {} {} from {} to {}", 
                  transaction.value, transaction.token_symbol.as_ref().unwrap_or(&"TRX".to_string()), 
                  transaction.from_address, transaction.to_address);
            
            // 发送大额转账通知
            if let Some(sender) = &self.notification_sender {
                let event = TransactionEvent {
                    transaction: transaction.clone(),
                    event_type: "large_transfer".to_string(),
                };
                
                if let Err(e) = sender.send(event) {
                    warn!("Failed to send large transfer event: {}", e);
                }
            }
        }

        Ok(())
    }

    /// 检查是否是大额转账
    fn is_large_transfer(&self, transaction: &Transaction) -> bool {
        // 解析金额
        if let Ok(amount) = transaction.value.parse::<f64>() {
            // 根据代币类型设置不同的阈值
            let threshold = match transaction.token_symbol.as_ref().map(|s| s.as_str()).unwrap_or("TRX") {
                "USDT" => 10000.0, // 10,000 USDT
                "TRX" => 1000000.0, // 1,000,000 TRX
                _ => 100000.0, // 默认阈值
            };
            
            amount >= threshold
        } else {
            false
        }
    }

    /// 计算扫描速度
    async fn calculate_scan_speed(&self, last_block_count: &mut u64, last_check: &mut std::time::Instant) {
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
    pub async fn reset(&mut self, start_block: Option<u64>) -> Result<()> {
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

    /// 获取扫描统计信息
    pub async fn get_statistics(&self) -> Result<ScannerStatistics> {
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
    pub async fn scan_specific_block(&mut self, block_number: u64) -> Result<u64> {
        info!("Manually scanning block {}", block_number);
        self.scan_block(block_number).await
    }

    /// 获取节点健康状态
    pub async fn get_node_health(&mut self) -> Result<bool> {
        self.tron_client.health_check().await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::*;

    #[tokio::test]
    async fn test_scanner_creation() {
        let config = Config::default();
        let db = Database::new(&config.database).await.unwrap();
        
        let scanner = Scanner::new(config, db);
        assert!(scanner.is_ok());
    }

    #[tokio::test]
    async fn test_large_transfer_detection() {
        let config = Config::default();
        let db = Database::new(&config.database).await.unwrap();
        let scanner = Scanner::new(config, db).unwrap();

        let transaction = Transaction {
            id: uuid::Uuid::new_v4(),
            hash: "test_hash".to_string(),
            block_number: 12345,
            block_hash: "test_block_hash".to_string(),
            transaction_index: 0,
            from_address: "from_addr".to_string(),
            to_address: "to_addr".to_string(),
            value: "15000.0".to_string(),
            token_address: None,
            token_symbol: Some("USDT".to_string()),
            token_decimals: Some(6),
            gas_used: Some(21000),
            gas_price: Some("20".to_string()),
            status: TransactionStatus::Success,
            timestamp: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert!(scanner.is_large_transfer(&transaction));
    }
}

