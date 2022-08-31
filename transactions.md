# Types of Transactions

## Deposit

A deposit is a credit to the client's asset account, meaning it should increase the available and total funds of the client account.

A deposit looks like this:

| type | client | tx | amount |
| --- | --- | --- | --- |
| deposit | 1 | 1 | 1.0 |

## Withdrawal

A withdraw is a debit to the client's asset account, meaning it should decrease the available and total funds of the client account.

A withdrawal looks like this:

| type | client | tx | amount |
| --- | --- | --- | --- |
| withdrawal | 2 |  2 | 1.0 |

If a client does not have sufficient available funds the withdrawal should fail and the total amount of funds should not change.

## Dispute

A dispute represents a client's claim that a transaction was erroneous and should be reversed. The transaction shouldn't be reversed yet but the associated funds should be held. This means that the clients available funds should decrease by the amount disputed, their held funds should increase by the amount disputed, while their total funds should remain the same.

A dispute looks like this:

| type | client | tx | amount |
| --- | --- | --- | --- |
|dispute | 1|  1 | |

Notice that a dispute does not state the amount disputed. Instead a dispute references the transaction that is disputed by ID. If the tx specified by the dispute doesn't exist we can ignore it and assume this is an error on our partners side.

## Resolve

A resolve represents a resolution to a dispute, releasing the associated held funds. Funds that were previously disputed are no longer disputed. This means that the clients held funds should decrease by the amount no longer disputed, their available funds should increase by the amount no longer disputed, and their total funds should remain the same.

A resolve looks like this:

| type | client | tx | amount |
| --- | --- | --- | --- |
| resolve | 1 | 1 | |

Like disputes, resolves do not specify an amount. Instead they refer to a transaction that was under dispute by ID. If the tx specified doesn't exist, or the tx isn't under dispute, we can ignore the resolve and assume this is an error on our partner's side.

## Chargeback

A chargeback is the final state of a dispute and represents the client reversing a transaction. Funds that were held have now been withdrawn. This means that the clients held funds and total funds should decrease by the amount previously disputed. If a chargeback occurs the client's account should be immediately frozen.

A chargeback looks like this:

| type | client | tx | amount |
| --- | --- | --- | --- |
| chargeback | 1 | 1 | |  

Like a dispute and a resolve a chargeback refers to the transaction by ID (tx) and does not specify an amount. Like a resolve, if the tx specified doesn't exist, or the tx isn't under dispute, we can ignore chargeback and assume this is an error on our partner's side.
