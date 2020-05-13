# rubot: alpha_beta

The goal of the `Bot` is to take a decision tree and output
the optimal decision.

At each depth of this tree we either want to maximise or minimize
the resulting fitness.

In a competitive two player game, maximizing depths are the turns in which the bot
is active, and during minimizing depths the opponent is able to make a decision.

Note that we do not expect a specific order of minimizing and maximizing turns.
It is for example completely valid to have a game in which some actions allow additional actions
for a player.

Let's first introduce the notation used from now on:

- `a n` := the bot is active, meaning we want to take the path which results in the highest fitness.
    the node itself has a fitness of `n`.
- `o n` := the opponent is active, meaning that we want to achieve the lowest possible fitness.

Each level of indentatation represents one action.
```
- a 0
  - o 7
  - o 3
```
The above tree means that the bot is active and has the choice between the action `[0]`,
with a fitness of `7` and `[1]` with a fitness of 3. To maximise the fitness, `[0]` would be chosen.

Paths are represented as a list of indices.

## Examples

```
- a 0
  - a 12 # child 0
  - a 7  # child 1

optimal action: [0], with fitness: 12
```

If the bot is able to analyse the complete decision tree, only the leaf nodes are relevant.

```
- a 0
  - a 12
    - a 6
  - a 7

optimal action: [1], with fitness: 7
```

While the bot wants to maximise the achieved fitness, the opponent is expected
to choose the path with the worst fitness.
```
- a 0
  - o 12
    - a 3 # The opponent would choose this, changing [0] to have a fitness of 3
    - a 7
  - a 6

optimal action: [1], with fitness: 6
```

## Cutoffs

Unlike the brute bot, alpha beta pruning allows us to skip subtrees if
it is provable that they always result in a less desirable outcome than an already computed
path.

```
- a 0
  - a 6
  - o 12
    - a 3
    - a 7
```
Walking through this decision tree from top to bottom, we know by visiting `[0]` that
there is a path with the fitness `6`. While analysing `[1]`, we realise that
our opponent could chose `[1][0]`, which would result in a fitness of `3`.

As `3` is smaller than `6`, we can stop looking at `[1]` entirely, without having seen `[1][1]`.

It is also possible to apply this in the other direction:

```
- a 0
 - o 0
   - a 6
   - a 0
     - a 7
     - a 3
```

While analysing `[0]`, the opponent first looks at `[0][0]` which ends up with a fitness of `6`.
Once they see `[0][1][0]` with a fitness of `7`, they known that we would always be able to choose
a path at `[0][1]` with a fitness `> 6`. This means that they are also able to skip `[0][1][1]` entirely.

- `alpha`: the best already certainly possible fitness at the current depth.
- `beta`: the worst already certainly achievable fitness by the opponent.
- `alpha cutoff`: skips paths where the opponent could choose an option which is smaller than `alpha`.
- `beta cutoff`: while minimizing, skips paths where we could choose an option better than `beta`.

It is of note that `alpha` can only be changed while maximizing, and `beta` is only
changed while minimizing.

Let's look at the actual `alpha` and `beta` values while we traverse the above tree depth first:

```
- a 0
 - o 0
   - a 6     1. alpha: None      beta: Some(6)
   - a 0
     - a 7   2. alpha: Some(7)   beta: Some(6) => cutoff
     - a 3   3. skipped
```

Both alpha and beta do not propagate upwards however.

TODO: Add an example here, improve examples in general.

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

The fitness of the best fully terminated path is used as the initial `alpha` value of future iterations.
This means that all partially terminated paths with a maximum fitness less than this `alpha` value can be ignored.
This is implemented in `fn add_complete` and `fn add_partial` of `struct Terminated`.
