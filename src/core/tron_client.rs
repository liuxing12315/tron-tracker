// GetBlock.io Tron API 客户端
//
// 负责与 GetBlock.io API 交互，获取 Tron 区块链数据

use crate::core::config::TronConfig;
use crate::core::models::*;
use anyhow::{anyhow, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client,
};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tracing::{debug, info, warn};
// Removed unused import: tokio::time::sleep

/// GetBlock.io API 客户端
#[derive(Debug)]
pub struct TronClient {
    client: Client,
    config: TronConfig,
    current_node_index: AtomicUsize,
}

impl Clone for TronClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
            current_node_index: AtomicUsize::new(self.current_node_index.load(Ordering::Relaxed)),
        }
    }
}

impl TronClient {
    /// 创建新的客户端实例
    pub fn new(config: TronConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()?;

        Ok(Self {
            client,
            config,
            current_node_index: AtomicUsize::new(0),
        })
    }

    /// 获取当前使用的节点 URL
    fn get_current_node_url(&self) -> &str {
        let index = self.current_node_index.load(Ordering::Relaxed);
        &self.config.nodes[index].url
    }

    /// 获取当前节点的 API Key
    fn get_current_api_key(&self) -> Option<&str> {
        let index = self.current_node_index.load(Ordering::Relaxed);
        self.config.nodes[index].api_key.as_deref()
    }

    /// 切换到下一个节点
    fn switch_to_next_node(&self) {
        let current = self.current_node_index.load(Ordering::Relaxed);
        let next = (current + 1) % self.config.nodes.len();
        self.current_node_index.store(next, Ordering::Relaxed);
        warn!("Switched to node: {}", self.get_current_node_url());
    }

    /// 发送 JSON-RPC 请求
    async fn send_request(&self, method: &str, params: Value) -> Result<Value> {
        let mut attempts = 0;
        let max_attempts = self.config.nodes.len() * 2; // 每个节点尝试2次

        loop {
            if attempts >= max_attempts {
                return Err(anyhow!("All nodes failed after {} attempts", attempts));
            }

            let url = format!("{}/jsonrpc", self.get_current_node_url());
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", HeaderValue::from_static("application/json"));

            // 如果有 API Key，添加到请求头
            if let Some(api_key) = self.get_current_api_key() {
                let auth_value = HeaderValue::from_str(&format!("Bearer {}", api_key))?;
                headers.insert(AUTHORIZATION, auth_value);
            }

            let request_body = json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
                "id": 1
            });

            debug!("Sending request to {}: {}", url, request_body);

            match self
                .client
                .post(&url)
                .headers(headers)
                .json(&request_body)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<Value>().await {
                            Ok(json_response) => {
                                if let Some(error) = json_response.get("error") {
                                    warn!("RPC error from {}: {}", url, error);
                                    attempts += 1;
                                    self.switch_to_next_node();
                                    continue;
                                }

                                if let Some(result) = json_response.get("result") {
                                    return Ok(result.clone());
                                } else {
                                    warn!("No result field in response from {}", url);
                                    attempts += 1;
                                    self.switch_to_next_node();
                                    continue;
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse JSON response from {}: {}", url, e);
                                attempts += 1;
                                self.switch_to_next_node();
                                continue;
                            }
                        }
                    } else {
                        warn!("HTTP error from {}: {}", url, response.status());
                        attempts += 1;
                        self.switch_to_next_node();
                        continue;
                    }
                }
                Err(e) => {
                    warn!("Request failed to {}: {}", url, e);
                    attempts += 1;
                    self.switch_to_next_node();
                    continue;
                }
            }
        }
    }

    /// 获取最新区块号
    pub async fn get_latest_block_number(&self) -> Result<u64> {
        let result = self.send_request("eth_blockNumber", json!([])).await?;

        let block_number_str = result
            .as_str()
            .ok_or_else(|| anyhow!("Invalid block number format"))?;

        // 移除 "0x" 前缀并解析十六进制
        let block_number = u64::from_str_radix(block_number_str.trim_start_matches("0x"), 16)?;

        debug!("Latest block number: {}", block_number);
        Ok(block_number)
    }

    /// 根据区块号获取区块数据
    pub async fn get_block_by_number(&self, block_number: u64) -> Result<BlockData> {
        let block_hex = format!("0x{:x}", block_number);
        let result = self
            .send_request("eth_getBlockByNumber", json!([block_hex, true]))
            .await?;

        self.parse_block_data(result).await
    }

    /// 根据区块哈希获取区块数据
    pub async fn get_block_by_hash(&self, block_hash: &str) -> Result<BlockData> {
        let result = self
            .send_request("eth_getBlockByHash", json!([block_hash, true]))
            .await?;

        self.parse_block_data(result).await
    }

    /// 解析区块数据
    async fn parse_block_data(&self, block_json: Value) -> Result<BlockData> {
        let block_obj = block_json
            .as_object()
            .ok_or_else(|| anyhow!("Invalid block data format"))?;

        // 解析区块号
        let number_str = block_obj
            .get("number")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing block number"))?;
        let block_number = u64::from_str_radix(number_str.trim_start_matches("0x"), 16)?;

        // 解析区块哈希
        let block_hash = block_obj
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing block hash"))?
            .to_string();

        // 解析父区块哈希
        let parent_hash = block_obj
            .get("parentHash")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 解析时间戳
        let timestamp_str = block_obj
            .get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing timestamp"))?;
        let timestamp = u64::from_str_radix(timestamp_str.trim_start_matches("0x"), 16)?;

        // 解析交易列表
        let transactions_array = block_obj
            .get("transactions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("Missing transactions array"))?;

        let mut transactions = Vec::new();
        for tx_value in transactions_array {
            if let Ok(transaction) = self
                .parse_transaction_data(tx_value.clone(), block_number, timestamp)
                .await
            {
                transactions.push(transaction);
            }
        }

        // info!("Parsed block {} with {} transactions", block_number, transactions.len());

        Ok(BlockData {
            number: block_number,
            hash: block_hash,
            parent_hash,
            timestamp,
            transaction_count: transactions.len() as u32,
            transactions,
        })
    }

    /// 解析交易数据
    async fn parse_transaction_data(
        &self,
        tx_json: Value,
        block_number: u64,
        block_timestamp: u64,
    ) -> Result<Transaction> {
        let tx_obj = tx_json
            .as_object()
            .ok_or_else(|| anyhow!("Invalid transaction data format"))?;

        // 解析交易哈希
        let hash = tx_obj
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing transaction hash"))?
            .to_string();

        // 解析发送方地址
        let from_address = tx_obj
            .get("from")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 解析接收方地址
        let to_address = tx_obj
            .get("to")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 解析交易金额
        let value_str = tx_obj
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");
        let value_wei = u128::from_str_radix(value_str.trim_start_matches("0x"), 16).unwrap_or(0);
        let amount = format!("{}", value_wei);

        // 解析 Gas 使用量
        let gas_used_str = tx_obj.get("gas").and_then(|v| v.as_str()).unwrap_or("0x0");
        let gas_used = u64::from_str_radix(gas_used_str.trim_start_matches("0x"), 16).unwrap_or(0);

        // 解析 Gas 价格
        let gas_price_str = tx_obj
            .get("gasPrice")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0");
        let gas_price =
            u64::from_str_radix(gas_price_str.trim_start_matches("0x"), 16).unwrap_or(0);

        // 确定代币类型和状态
        let (token, status) = self.determine_token_and_status(&tx_obj).await;

        // 创建时间戳
        let timestamp = chrono::DateTime::from_timestamp(block_timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now());

        Ok(Transaction {
            id: uuid::Uuid::new_v4(),
            hash,
            block_number: block_number,
            block_hash: "".to_string(), // 需要从区块数据中获取
            transaction_index: 0,       // 需要从交易索引中获取
            from_address,
            to_address,
            value: amount,
            token_address: tx_obj
                .get("to")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            token_symbol: Some(token.clone()),
            token_decimals: if token == "USDT" { Some(6) } else { Some(6) },
            gas_used: Some(gas_used as u64),
            gas_price: Some(gas_price.to_string()),
            status,
            timestamp,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    /// 确定代币类型和交易状态
    async fn determine_token_and_status(
        &self,
        tx_obj: &serde_json::Map<String, Value>,
    ) -> (String, TransactionStatus) {
        // 检查是否是合约调用
        if let Some(input) = tx_obj.get("input").and_then(|v| v.as_str()) {
            if input.len() > 2 && input != "0x" {
                // 这是一个合约调用，可能是 TRC20 代币转账
                if input.starts_with("0xa9059cbb") {
                    // transfer(address,uint256) 方法签名
                    return ("USDT".to_string(), TransactionStatus::Success);
                }
            }
        }

        // 检查交易状态
        let status = if let Some(status_str) = tx_obj.get("status").and_then(|v| v.as_str()) {
            if status_str == "0x1" {
                TransactionStatus::Success
            } else {
                TransactionStatus::Failed
            }
        } else {
            TransactionStatus::Pending
        };

        // 默认为 TRX
        ("TRX".to_string(), status)
    }

    /// 获取交易收据
    pub async fn get_transaction_receipt(&self, tx_hash: &str) -> Result<Value> {
        self.send_request("eth_getTransactionReceipt", json!([tx_hash]))
            .await
    }

    /// 获取账户余额
    pub async fn get_balance(&self, address: &str) -> Result<String> {
        let result = self
            .send_request("eth_getBalance", json!([address, "latest"]))
            .await?;

        let balance_str = result
            .as_str()
            .ok_or_else(|| anyhow!("Invalid balance format"))?;

        let balance_wei = u128::from_str_radix(balance_str.trim_start_matches("0x"), 16)?;
        Ok(format!("{}", balance_wei))
    }

    /// 获取 TRC20 代币余额
    pub async fn get_token_balance(
        &self,
        contract_address: &str,
        wallet_address: &str,
    ) -> Result<String> {
        // 构造 balanceOf 方法调用
        let method_signature = "70a08231"; // balanceOf(address)
        let padded_address = format!("{:0>64}", wallet_address.trim_start_matches("0x"));
        let data = format!("0x{}{}", method_signature, padded_address);

        let call_params = json!([{
            "to": contract_address,
            "data": data
        }, "latest"]);

        let result = self.send_request("eth_call", call_params).await?;

        let balance_str = result
            .as_str()
            .ok_or_else(|| anyhow!("Invalid token balance format"))?;

        let balance = u128::from_str_radix(balance_str.trim_start_matches("0x"), 16)?;
        Ok(format!("{}", balance))
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<bool> {
        match self.get_latest_block_number().await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Removed TronNodeConfig import - using NodeConfig directly

    #[tokio::test]
    async fn test_tron_client_creation() {
        let config = TronConfig {
            nodes: vec![crate::core::config::NodeConfig {
                name: "TronGrid".to_string(),
                url: "https://api.trongrid.io".to_string(),
                api_key: None,
                priority: 1,
                timeout: 30,
            }],
            api_key: None,
            timeout: 30,
        };

        let client = TronClient::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_get_latest_block_number() {
        // 这个测试需要实际的网络连接
        // 在实际环境中可以启用
        /*
        let config = TronConfig {
            nodes: vec![TronNodeConfig {
                url: "https://api.trongrid.io".to_string(),
                api_key: None,
                priority: 1,
                enabled: true,
            }],
            timeout: 30,
        };

        let mut client = TronClient::new(config).unwrap();
        let block_number = client.get_latest_block_number().await;
        assert!(block_number.is_ok());
        assert!(block_number.unwrap() > 0);
        */
    }
}
