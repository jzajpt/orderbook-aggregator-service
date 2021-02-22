# Keyrock Challenge

This repo contains a solution to the Keyrock Rust hiring challenge.

## Problem statement

Using Rust, code a mini project that:
1. connects to two exchange’s websocket feeds at the same time,
2. pulls order book s, using these streaming connections, f or a given traded pair of currencies (configurable), from each exchange,
3. merges and sorts the order books to create a combined order book,
4. from the combined book, publishes the spread , top ten bids , and top ten asks , as a stream, through a gRPC server.

## Solution

Solution will be build wth async Rust (using `tokio` runtime). For each exchange
an async task will be spawn that will be handling the connection and processing
the stream.

Sharing data between concurrent tasks will be done using passing messages using
channel primitives provided by `tokio`.

### Binance

Binance doesn't offer real-time stream for order book changes, so the solution
will use their "Partial Book Depth Stream" which provides updates each 100ms.

### Bitstamp

Bitstamp websocket API offers live order book endpoint which is streaming top
100 bids and asks real-time.

## TODO

- [x] Combined orderbook
- [ ] Connection reset handling
- [ ] CTRL+C graceful handling
- [ ] gRPC interface - server
- [ ] gRPC client