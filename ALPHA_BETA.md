# rubot: alpha_beta

The goal of the `Bot` is to take a decision tree and output
the optimal decision.

At each depth of this tree we either want to maximise or minimize
the resulting fitness.

In a competitive two player game, maximizing depths are the turns in which the bot
is active, and during minimizing depths the opponent is able to make a decision.

- `a n` := the bot is active, meaning we want to take the path which results in the highest fitness.
    the node itself has a fitness of `n`.
- `o n` := the opponent is active, meaning that we want to achieve the lowest possible fitness.

Paths are represented as a list of indices.

## Examples

```
a 0
  - a 12 # child 0
  - a 7  # child 1

optimal action: [0], with fitness: 12

a 0
  - a 12
    - a 6
  - a 7

optimal action: [1], with fitness: 7

a 0
  - o 12
    - a 3
    - a 7
  - a 6

optimal action: [1], with fitness: 6
```

## Cutoffs

Unlike the brute bot, alpha beta pruning allows us to skip subtrees if
it is provable that they always result less desirable outcome than an already computed
path.

```
a 0
  - a 6
  - o 12
    - a 3
    - a 7
```
Walking through this decision tree from top to bottom, we know by visiting `[0]` that
there is a path with the fitness `6`. While analysing `[1]`, we realise that
our opponent could chose `[1][0]`, which would result in a fitness of `3`.

As `3` is worse than `6`, we can stop looking at `[1]` entirely, without having seen `[1][1]`.

It is also possible to apply this in the other direction:

```
a 0
 - o 0
   - a 6
   - a 0
     - a 7
     - a 3
```

While analysing `[0]`, the opponent first looks at `[0][0]` which ends up with a fitness of `6`.
Once they see `[0][1][0]` with a fitness of `7`, they known that we would always be able to choose
a path at `[0][1]` with a fitness `> 6`. This means that they are also able to skip `[0][1][1]` entirely.

- `alpha`: the worst guaranteed fitness.
- `beta`: the currently best value, which cannot be denied by the opponent.
- `alpha cutoff`: skips paths where the opponent could choose an option which is worse than `alpha`.
- `beta cutoff`: while minimizing, skips paths where we could choose an option better than `beta`.

## Other optimizations

`alpha_beta` uses iterative deepening, meaning that it first only checks the next moves directly,
then it looks two moves ahead, then three and so on.

This makes it easier to run under some time constraint, as if we are unable to finish
an iteration in time, we can always use the data from previous, shallower iterations.

It is also faster, as we can use the knowledge of previous iterations to adapt the order of our tests,
which hugely influences the branches we can skip due to a cutoff.

The current implementation only remembers the *best* path of the previous depth.
It may be advantageous to also store the best path the opponent can take for all other options,
which may greatly improve the amount of possible cutoffs.


In case we were able to fully analyse an arm of the decision tree at depth `n`, we can use its fitness in all following
iterations without having to recompute this possibility.

More interestingly, we can have arms which did not reach the current depth `n` and ignored sections due to an cutoff. As the `alpha` and
`beta` values are often based on incomplete data, it is possible that the ignored section at depth `n` may be the best option at depth
`n + 1`.

We therefore store all partially terminated arms with their best possible fitness, and only retest them in future iterations if their
best possible fitness is greater than the current `alpha` value.

As the use the fitness of the best fully terminated paths as the `alpha` value of future iterations, all partially terminated paths for
which the best possible fitness is worse than this `alpha` can be ignored.
