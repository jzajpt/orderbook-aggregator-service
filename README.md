# Orderbook Aggregator

This repo contains a solution to the Keyrock Rust challenge.

## Problem statement

Using Rust, code a mini project that:
1. connects to two exchange’s websocket feeds at the same time,
2. pulls order books, using these streaming connections, f or a given traded pair of currencies (configurable), from each exchange,
3. merges and sorts the order books to create a combined order book,
4. from the combined book, publishes the spread , top ten bids , and top ten asks , as a stream, through a gRPC server.

## How to run this

If you have `grpcurl` installed, you can test the version deployed on
DigitalOcean:

```
grpcurl -plaintext -import-path ./proto -proto orderbook.proto \ 
161.35.221.121:50051 orderbook.OrderbookAggregator/BookSummary
```

## Architecture

The aggregator is built using `tokio` async runtime with `websocket_lite` for
websocket handling and `tonic` for gRPC.

### Binance

Binance doesn't offer real-time stream for order book changes, so the solution
will use their "Partial Book Depth Stream" which provides updates each 100ms.

### Bitstamp

Bitstamp websocket API offers live order book endpoint which is streaming top
100 bids and asks real-time.

## TODO

- [x] Exchange connectors
- [x] Combined orderbook
- [x] gRPC interface - server
- [ ] CircleCI build
- [ } Deployment
- [ ] Invalid pair name handling
- [ ] Connection reset handling
- [ ] CTRL+C graceful handling
- [ ] gRPC client
- [ ] more tests


## Open questiosn

* How much "gracefully" handle error states?
* What should the aggregator return if only one of the exchanges is available?
* How to handle 
