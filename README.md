#! **ru**st **bot**

A rust library for easily usable game bots. While these controllers are a lot worse than a specialized engine like [Stockfish], they are often good enough be an interesting challenge in most games.

Integrating `rubot` in your project should be possible in about half an hour while requiring a total of less than **100** lines of code. 
If this is not the case for your project, please create an issue on github.

## Examples

To run the examples, download the repository and run `cargo run --example <example name>`.

- `tic-tac-toe`: A port of [Sunjay's wonderful tic-tac-toe implementation], adding and using `rubot` required about 40 loc.
- `oko`: A mixture of [Dots and Boxes] and tic-tac-toe, `rubot` took 30 additional loc.

[Stockfish]:https://www.chessprogramming.org/Stockfish
[Sunjay's wonderful implementation]: https://github.com/sunjay/tic-tac-toe.git
