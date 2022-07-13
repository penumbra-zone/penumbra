# Overview of the Poseidon Permutation

This section describes the Poseidon permutation. It consists of *rounds*,
where each round has the following steps:

* `AddRoundConstants`: where constants (denoted by `arc` in the code) are added
to the internal state,
* `SubWords`: where the S-box $S(x) = x^{\alpha}$ is applied to the internal state,
* `MixLayer`: where a matrix is multiplied with the internal state.

The total number of rounds we denote by $R$. There are two types of round in the
Poseidon construction, partial and full. We denote the number of partial and
full rounds by $R_P$ and $R_F$ respectively.

In a full round in the `SubWords` layer the S-box is applied to each element of the
internal state, as shown in the diagram below:

```
  ┌───────────────────────────────────────────────────────────┐
  │                                                           │
  │                     AddRoundConstants                     │
  │                                                           │
  └────────────┬──────────┬──────────┬──────────┬─────────────┘
               │          │          │          │              
             ┌─▼─┐      ┌─▼─┐      ┌─▼─┐      ┌─▼─┐            
             │ S │      │ S │      │ S │      │ S │            
             └─┬─┘      └─┬─┘      └─┬─┘      └─┬─┘            
               │          │          │          │              
  ┌────────────▼──────────▼──────────▼──────────▼─────────────┐
  │                                                           │
  │                         MixLayer                          │
  │                                                           │
  └────────────┬──────────┬──────────┬──────────┬─────────────┘
               │          │          │          │              
               ▼          ▼          ▼          ▼              
```

In a partial round, in the `SubWords` layer we apply the S-box only to one element
of the internal state, as shown in the diagram below:

```                  
               │          │          │          │              
               │          │          │          │              
  ┌────────────▼──────────▼──────────▼──────────▼─────────────┐
  │                                                           │
  │                     AddRoundConstants                     │
  │                                                           │
  └────────────┬──────────────────────────────────────────────┘
               │                                               
             ┌─▼─┐                                             
             │ S │                                             
             └─┬─┘                                             
               │                                               
  ┌────────────▼──────────────────────────────────────────────┐
  │                                                           │
  │                         MixLayer                          │
  │                                                           │
  └────────────┬──────────┬──────────┬──────────┬─────────────┘
               │          │          │          │              
               ▼          ▼          ▼          ▼              
```

We apply half the full rounds ($R_f = R_F / 2$) first, then we apply the $R_P$ partial
rounds, then the rest of the $R_f$ full rounds. This is called the HADES design strategy in the literature.
