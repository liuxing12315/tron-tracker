-- Add start_block to system_config
INSERT INTO system_config (key, value, description)
VALUES (
    'start_block',
    '62800000',
    'The block number from which to start scanning. Can be adjusted manually.'
) ON CONFLICT (key) DO NOTHING;

-- Add blockchain nodes configuration
INSERT INTO system_config (key, value, description)
VALUES (
    'blockchain_nodes',
    '[
        {
            "name": "TronGrid",
            "url": "https://api.trongrid.io",
            "api_key": null,
            "priority": 1,
            "timeout": 30
        },
        {
            "name": "TronStack",
            "url": "https://api.tronstack.io",
            "api_key": null,
            "priority": 2,
            "timeout": 30
        },
        {
            "name": "GetBlock",
            "url": "https://go.getblock.io/tron",
            "api_key": null,
            "priority": 3,
            "timeout": 30
        }
    ]'::jsonb,
    'List of blockchain nodes for failover and load balancing. Can be updated dynamically.'
) ON CONFLICT (key) DO NOTHING;