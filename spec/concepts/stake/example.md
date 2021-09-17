# Example Staking Dynamics

To illustrate the dynamics of this system, consider a toy scenario with three
delegators, Alice, Bob, and Charlie, and two validators, Victoria and
William.  Tendermint consensus requires at least four validators and no one
party controlling more than $1/3$ of the stake, but this example uses only a few parties just to illustrate the dynamics.

For simplicity, the the base reward rates and commission rates
are fixed over all epochs at $r = 0.0006$ and $c_v = 0$, $c_w = 0.1$.
The `PEN` and `PENb` holdings of participant $a, b, c, \ldots$ are
denoted by $x_a$, $y_a$, etc., respectively.

Alice starts with $y_a = 10000$ `PENb` bonded to Victoria, Bob starts with $y_b = 10000$ `PENb` bonded to William, and Charlie starts with $x_c =  20000$ unbonded `PEN`.

- At genesis, Alice, Bob, and Charlie respectively have fractions $25\%$, $25\%$, and $50%$ of the total stake, and fractions $50\%$, $50\%$, $0\%$ of the total voting power.

- At epoch $e = 1$, Alice, Bob, and Charlie's holdings remain unchanged, but their unrealized notional values have changed.
    - Victoria charges zero commission, so $\psi_v(1) = \psi(1) = 1.0006$.  Alice's $y_a = 10000$ `PENb(v)` is now worth $10006$ `PEN`.
    - William charges $10\%$ commission, so $\psi_w(1) = 1.00054$.  Bob's $y_b = 10000$ `PENb(w)` is now worth $10005.4$, and William receives $0.6$ `PEN`.
    - William can use the commission to cover expenses, or self-delegate.  In this example, we assume that validators self-delegate their entire commission, to illustrate the staking dynamics.
    - William self-delegates $0.6$ `PEN`, to get $0.6 / \psi_w(2) = 0.6 / 1.00054^2 = 0.59935\ldots$ `PENb` in the next epoch, epoch $2$.

- At epoch $e = 90$:
    - Alice's $y_a = 10000$ `PENb(v)` is now worth $10554.67$ `PEN`.
    - Bob's $y_b = 10000$ `PENb(w)` is now worth $10497.86$ `PEN`.
    - William's self-delegation of accumulated commission has resulted in $y_w = 53.483$ `PENb(w)`.
    - Victoria's delegation pool remains at size $10000$ `PENb(v)`.  William's delegation pool has increased to $10053.483$ `PENb(w)`.  However, their respective adjustment factors are now $\theta_v(90) = 1$ and $\theta_w(90) = 0.99462$, so the voting powers of their delegation pools are respectively $10000$ and $9999.37$.
        - The slight loss of voting power for William's delegation pool occurs because William self-delegates rewards with a one epoch delay, thus missing one epoch of compounding.
    - Charlie's unbonded $x_c = 20000$ `PEN` remains unchanged, but its value relative to Alice and Bob's stake has declined.  
    - William's commission transfers stake from Bob, whose voting power has slightly declined relative to Alice's.
    - The distribution of stake between Alice, Bob, Charlie, and William is now $25.67\%$, $25.54\%$, $48.65\%$, $0.14\%$ respectively.  The distribution of voting power is $50\%$, $49.74\%$, $0\%$, $0.27\%$ respectively.
    - Charlie decides to bond his stake, split evenly between Victoria and William, to get $10000 / \psi_v(91) = 9485.85$ `PENb(v)` and $10000 / \psi_w(91) = 9536$ `PENb(w)`.

- At epoch $e = 91$:
    - Charlie now has $9468.80$ `PENb(v)` and $9520.60$ `PENb(w)`, worth $20000$ `PEN`.
    - For the same amount of unbonded stake, Charlie gets more `PENb(w)` than `PENb(v)`, because the exchange rate $\psi_w$ prices in the cumulative effect of commission since genesis, but Charlie isn't charged for commission during the time he didn't delegate to William.
    - William's commission for this epoch is now $1.233$ `PEN`, up from $0.633$ `PEN` in the previous epoch.
    - The distribution of stake between Alice, Bob, Charlie, and William is now $25.68\%$, $25.54\%$, $48.64\%$, $0.14\%$ respectively.  Because all stake is now bonded, except William's commission for this epoch, which is insignificant, the distribution of voting power is identical to the distribution of stake.

- At epoch $e = 180$:
    - Alice's $y_a = 10000$ `PENb(v)` is now worth $11140.12$ `PEN`.
    - Bob's $y_b = 10000$ `PENb(w)` is now worth $11020.52$ `PEN`.
    - Charlies's $y_{c,v} = 9468.80$ `PENb(v)` is now worth $10548.37$ `PEN`, and his $y_{c,w} = 9520.60$ `PENb(w)` is now worth $10492.20$ `PEN`.
    - William's self-delegation of accumulated commission has resulted in $y_w = 158.77$ `PENb(w)`, worth $176.30$ `PEN`.
    - The distribution of stake and voting power between Alice, Bob, Charlie, and William is now $25.68\%$, $25.41\%$, $48.51\%$, $0.40\%$ respectively.

This scenario was generated with a model in [this Google Sheet](https://docs.google.com/spreadsheets/d/1xUroRBT4rL9KumRbKVvxmkyC5m1zqCaaQWoeZv4P5PA/edit?usp=sharing).
