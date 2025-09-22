# CC Chain API Reference

## Overview

CC Chain provides multiple API interfaces for interacting with the blockchain:

- **JSON-RPC API**: Standard blockchain RPC interface
- **REST API**: RESTful HTTP endpoints for web applications
- **WebSocket API**: Real-time event streaming
- **CLI Interface**: Command-line tools for all operations

## JSON-RPC API

The JSON-RPC API follows the standard JSON-RPC 2.0 specification and is available at the node's RPC endpoint (default: `http://localhost:8001`).

### Base URL
```
http://localhost:8001/rpc
```

### Authentication

Most read operations require no authentication. Write operations require signed transactions.

### Error Handling

Errors follow the JSON-RPC 2.0 error format:

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Transaction failed",
    "data": {
      "reason": "Insufficient balance",
      "required": "1000000",
      "available": "500000"
    }
  },
  "id": 1
}
```

## Blockchain Methods

### `chain_getLatestBlock`

Get the latest block in the chain.

#### Parameters
None

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "header": {
      "prev_hash": "0x123...",
      "height": 12345,
      "timestamp": 1701234567890,
      "proposer": "0xabc...",
      "tx_merkle_root": "0x456...",
      "state_root": "0x789...",
      "gas_used": 150000,
      "gas_limit": 1000000
    },
    "transactions": [
      {
        "hash": "0xdef...",
        "from": "0x111...",
        "to": "0x222...",
        "amount": 1000000,
        "fee": 1000,
        "nonce": 5,
        "gas_limit": 21000,
        "signature": "0x333..."
      }
    ]
  },
  "id": 1
}
```

### `chain_getBlock`

Get a specific block by height or hash.

#### Parameters
- `block_identifier` (string): Block height (number) or hash (0x-prefixed hex)

#### Example Request
```json
{
  "jsonrpc": "2.0",
  "method": "chain_getBlock",
  "params": ["12345"],
  "id": 1
}
```

### `chain_getBlockByHash`

Get a block by its hash.

#### Parameters
- `hash` (string): Block hash (0x-prefixed hex)

#### Example Request
```json
{
  "jsonrpc": "2.0",
  "method": "chain_getBlockByHash",
  "params": ["0x123..."],
  "id": 1
}
```

### `chain_getChainInfo`

Get general chain information.

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "chain_id": "cc-mainnet",
    "latest_height": 12345,
    "latest_block_hash": "0x123...",
    "total_transactions": 98765,
    "validator_count": 21,
    "total_stake": "10000000000",
    "network_version": "1.0.0"
  },
  "id": 1
}
```

## Transaction Methods

### `tx_sendTransaction`

Submit a signed transaction to the network.

#### Parameters
- `transaction` (object): Signed transaction object

#### Example Request
```json
{
  "jsonrpc": "2.0",
  "method": "tx_sendTransaction",
  "params": [{
    "from": "0x111...",
    "to": "0x222...",
    "amount": "1000000",
    "fee": "1000",
    "nonce": 5,
    "gas_limit": 21000,
    "signature": "0x333..."
  }],
  "id": 1
}
```

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "transaction_hash": "0xdef...",
    "status": "pending"
  },
  "id": 1
}
```

### `tx_getTransaction`

Get transaction details by hash.

#### Parameters
- `hash` (string): Transaction hash (0x-prefixed hex)

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "hash": "0xdef...",
    "block_height": 12345,
    "block_hash": "0x123...",
    "transaction_index": 0,
    "from": "0x111...",
    "to": "0x222...",
    "amount": "1000000",
    "fee": "1000",
    "nonce": 5,
    "gas_limit": 21000,
    "gas_used": 21000,
    "status": "success",
    "signature": "0x333..."
  },
  "id": 1
}
```

### `tx_getTransactionReceipt`

Get transaction receipt with execution details.

#### Parameters
- `hash` (string): Transaction hash

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "transaction_hash": "0xdef...",
    "block_height": 12345,
    "block_hash": "0x123...",
    "transaction_index": 0,
    "status": "success",
    "gas_used": 21000,
    "logs": [],
    "events": []
  },
  "id": 1
}
```

### `tx_estimateGas`

Estimate gas required for a transaction.

#### Parameters
- `transaction` (object): Unsigned transaction object

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "gas_estimate": 21000,
    "gas_price": 50
  },
  "id": 1
}
```

## Account Methods

### `account_getBalance`

Get account balance.

#### Parameters
- `address` (string): Account address (0x-prefixed hex)
- `block_height` (string, optional): Block height ("latest" if omitted)

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "balance": "1000000000",
    "nonce": 42
  },
  "id": 1
}
```

### `account_getNonce`

Get account nonce (transaction count).

#### Parameters
- `address` (string): Account address

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "nonce": 42
  },
  "id": 1
}
```

### `account_getTransactionHistory`

Get transaction history for an account.

#### Parameters
- `address` (string): Account address
- `limit` (number, optional): Maximum number of transactions (default: 50)
- `offset` (number, optional): Pagination offset (default: 0)

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "transactions": [
      {
        "hash": "0xdef...",
        "from": "0x111...",
        "to": "0x222...",
        "amount": "1000000",
        "fee": "1000",
        "block_height": 12345,
        "timestamp": 1701234567890,
        "status": "success"
      }
    ],
    "total_count": 150,
    "has_more": true
  },
  "id": 1
}
```

## Smart Contract Methods

### `contract_call`

Call a smart contract function (read-only).

#### Parameters
- `contract_address` (string): Contract address
- `function_name` (string): Function to call
- `args` (array): Function arguments
- `block_height` (string, optional): Block height for state query

#### Example Request
```json
{
  "jsonrpc": "2.0",
  "method": "contract_call",
  "params": [
    "0xcontract123...",
    "getBalance",
    ["0xuser456..."],
    "latest"
  ],
  "id": 1
}
```

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "return_value": "1000000",
    "gas_used": 5000
  },
  "id": 1
}
```

### `contract_deploy`

Deploy a new smart contract.

#### Parameters
- `bytecode` (string): Contract bytecode (hex)
- `constructor_args` (array): Constructor arguments
- `deployer` (string): Deployer address
- `gas_limit` (number): Gas limit for deployment

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "contract_address": "0xnewcontract...",
    "transaction_hash": "0xdeployment...",
    "gas_used": 500000
  },
  "id": 1
}
```

## Validator Methods

### `validator_getValidators`

Get current validator set.

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "validators": [
      {
        "public_key": "0xvalidator1...",
        "stake": "1000000000",
        "performance_score": 0.98,
        "last_activity": 1701234567890,
        "status": "active"
      }
    ],
    "total_stake": "10000000000",
    "byzantine_threshold": "3333333333"
  },
  "id": 1
}
```

### `validator_getPerformanceMetrics`

Get validator performance metrics.

#### Parameters
- `validator_address` (string): Validator public key

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "blocks_proposed": 150,
    "blocks_validated": 5000,
    "uptime_percentage": 98.5,
    "average_response_time": 250,
    "slashing_events": 0,
    "performance_score": 0.98
  },
  "id": 1
}
```

## Network Methods

### `network_getPeers`

Get connected peers.

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "peers": [
      {
        "id": "peer123...",
        "address": "192.168.1.100:8000",
        "node_type": "validator",
        "version": "1.0.0",
        "latency": 50,
        "connected_since": 1701234567890
      }
    ],
    "peer_count": 25
  },
  "id": 1
}
```

### `network_getNetworkInfo`

Get network information.

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "network_id": "cc-mainnet",
    "peer_count": 25,
    "max_peers": 50,
    "node_version": "1.0.0",
    "consensus_version": "ccbft-1.0",
    "sync_status": "synced"
  },
  "id": 1
}
```

## Bridge Methods

### `bridge_getSupportedChains`

Get supported bridge chains.

#### Returns
```json
{
  "jsonrpc": "2.0",
  "result": {
    "chains": [
      {
        "chain_id": "ethereum",
        "name": "Ethereum",
        "status": "active",
        "supported_assets": ["ETH", "USDC", "USDT"]
      },
      {
        "chain_id": "bitcoin",
        "name": "Bitcoin",
        "status": "active", 
        "supported_assets": ["BTC"]
      }
    ]
  },
  "id": 1
}
```

### `bridge_transfer`

Initiate a cross-chain transfer.

#### Parameters
- `from_chain` (string): Source chain ID
- `to_chain` (string): Destination chain ID
- `asset` (string): Asset to transfer
- `amount` (string): Transfer amount
- `destination_address` (string): Destination address
- `sender_signature` (string): Transfer authorization signature

## REST API

The REST API provides a more traditional HTTP interface for web applications.

### Base URL
```
http://localhost:8080/api/v1
```

### Authentication

Include API key in header for write operations:
```
Authorization: Bearer your-api-key
```

### Common Response Format

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "timestamp": 1701234567890
}
```

### Endpoints

#### GET `/chain/info`

Get chain information.

**Response:**
```json
{
  "success": true,
  "data": {
    "chain_id": "cc-mainnet",
    "latest_height": 12345,
    "latest_block_hash": "0x123...",
    "total_transactions": 98765
  }
}
```

#### GET `/blocks/latest`

Get the latest block.

#### GET `/blocks/{height}`

Get block by height.

#### GET `/blocks/hash/{hash}`

Get block by hash.

#### GET `/transactions/{hash}`

Get transaction by hash.

#### GET `/accounts/{address}/balance`

Get account balance.

**Response:**
```json
{
  "success": true,
  "data": {
    "address": "0x123...",
    "balance": "1000000000",
    "nonce": 42
  }
}
```

#### GET `/accounts/{address}/transactions`

Get account transaction history.

**Query Parameters:**
- `limit`: Number of transactions (default: 50)
- `offset`: Pagination offset (default: 0)
- `type`: Transaction type filter

#### POST `/transactions`

Submit a transaction.

**Request Body:**
```json
{
  "from": "0x111...",
  "to": "0x222...",
  "amount": "1000000",
  "fee": "1000",
  "nonce": 5,
  "signature": "0x333..."
}
```

#### GET `/validators`

Get current validator set.

#### GET `/validators/{address}/metrics`

Get validator performance metrics.

## WebSocket API

Real-time event streaming via WebSocket connection.

### Connection URL
```
ws://localhost:8080/ws
```

### Message Format

```json
{
  "type": "subscription",
  "event": "new_block",
  "data": { ... }
}
```

### Subscription Types

#### `new_block`

Subscribe to new blocks.

```json
{
  "type": "subscribe",
  "event": "new_block"
}
```

#### `new_transaction`

Subscribe to new transactions.

```json
{
  "type": "subscribe", 
  "event": "new_transaction"
}
```

#### `account_activity`

Subscribe to account activity.

```json
{
  "type": "subscribe",
  "event": "account_activity",
  "params": {
    "address": "0x123..."
  }
}
```

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| -32000 | Server Error | Generic server error |
| -32001 | Transaction Failed | Transaction execution failed |
| -32002 | Insufficient Balance | Account has insufficient balance |
| -32003 | Invalid Signature | Transaction signature is invalid |
| -32004 | Nonce Too Low | Transaction nonce is too low |
| -32005 | Gas Limit Exceeded | Transaction exceeds gas limit |
| -32006 | Block Not Found | Requested block does not exist |
| -32007 | Transaction Not Found | Requested transaction does not exist |
| -32008 | Invalid Address | Address format is invalid |
| -32009 | Contract Error | Smart contract execution error |
| -32010 | Network Error | Network connectivity error |

## Rate Limiting

API endpoints are rate limited to prevent abuse:

- **Public endpoints**: 1000 requests per minute per IP
- **Authenticated endpoints**: 5000 requests per minute per API key
- **WebSocket connections**: 100 connections per IP

Rate limit headers are included in responses:
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1701234567
```

## SDKs and Libraries

Official SDKs are available for popular programming languages:

- **JavaScript/TypeScript**: `npm install @cc-chain/sdk`
- **Python**: `pip install cc-chain-sdk`
- **Go**: `go get github.com/ccx404/cc-chain-go`
- **Rust**: `cargo add cc-chain-sdk`
- **Java**: Available via Maven Central

### JavaScript Example

```javascript
import { CCChainClient } from '@cc-chain/sdk';

const client = new CCChainClient('http://localhost:8001');

// Get latest block
const block = await client.getLatestBlock();
console.log('Latest block:', block.height);

// Send transaction
const keypair = CCChainClient.generateKeypair();
const tx = await client.sendTransaction({
  from: keypair.address,
  to: '0x222...',
  amount: '1000000',
  fee: '1000'
}, keypair);

console.log('Transaction hash:', tx.hash);
```

### Python Example

```python
from cc_chain_sdk import CCChainClient, Keypair

client = CCChainClient('http://localhost:8001')

# Get account balance
balance = client.get_balance('0x123...')
print(f'Balance: {balance}')

# Send transaction
keypair = Keypair.generate()
tx_hash = client.send_transaction(
    from_address=keypair.address,
    to_address='0x222...',
    amount=1000000,
    fee=1000,
    private_key=keypair.private_key
)
print(f'Transaction: {tx_hash}')
```

## Testing

Use the included test network for development:

```bash
# Start test network
cargo run --bin cc-node -- start \
  --node-type validator \
  --listen 127.0.0.1:8000 \
  --data-dir ./test-data \
  --test-network

# Connect to test network
curl -X POST http://localhost:8001/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "chain_getChainInfo",
    "id": 1
  }'
```

## Support

For API support and questions:

- **Documentation**: https://docs.cc-chain.org
- **Discord**: https://discord.gg/cc-chain  
- **GitHub Issues**: https://github.com/ccx404/cc-chain/issues
- **Email**: api-support@cc-chain.org