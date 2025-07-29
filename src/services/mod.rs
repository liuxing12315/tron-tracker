pub mod scanner;
pub mod webhook;
pub mod websocket;
pub mod notification;

use crate::core::{config::Config, database::Database};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ServiceManager {
    pub config: Config,
    pub db: Database,
    pub scanner: Arc<RwLock<scanner::Scanner>>,
    pub webhook_service: Arc<RwLock<webhook::WebhookService>>,
    pub websocket_service: Arc<RwLock<websocket::WebSocketService>>,
}

impl ServiceManager {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Database::new(&config.database).await?;
        
        let scanner = Arc::new(RwLock::new(scanner::Scanner::new(config.clone(), db.clone())));
        let webhook_service = Arc::new(RwLock::new(webhook::WebhookService::new(config.clone(), db.clone())));
        let websocket_service = Arc::new(RwLock::new(websocket::WebSocketService::new(config.clone())));
        
        Ok(Self {
            config,
            db,
            scanner,
            webhook_service,
            websocket_service,
        })
    }
    
    pub async fn start_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Start scanner
        {
            let scanner = self.scanner.read().await;
            scanner.start().await?;
        }
        
        // Start webhook service
        {
            let webhook_service = self.webhook_service.read().await;
            webhook_service.start().await?;
        }
        
        // Start websocket service
        {
            let websocket_service = self.websocket_service.read().await;
            websocket_service.start().await?;
        }
        
        Ok(())
    }
    
    pub async fn stop_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Stop all services gracefully
        {
            let scanner = self.scanner.read().await;
            scanner.stop().await?;
        }
        
        {
            let webhook_service = self.webhook_service.read().await;
            webhook_service.stop().await?;
        }
        
        {
            let websocket_service = self.websocket_service.read().await;
            websocket_service.stop().await?;
        }
        
        Ok(())
    }
}

