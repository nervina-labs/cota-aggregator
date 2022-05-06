## aggregator-tests

The test include three parts: one mint transaction and two transfer transactions and the transactions will run in three different threads.

```shell
                 Mint NFT1 and NFT2 
1. Issuer ----------------------------------> receiver1 and receiver2

                 Transfer NFT1              
2. Receiver1 -----------------------> receiver2 

                 Transfer NFT2              
3. Receiver2 -----------------------> receiver1
```

These three transactions are interdependent and the data is in a competitive relationship, so the lock is need in aggregator server.