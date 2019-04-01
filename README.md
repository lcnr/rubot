#! game-controller

A rust library for easily usable game controllers. While these controllers are a lot worse than a specialized engine, like [Stockfish]. They are often good enough be a challenge in most games.

Integrating `game-controller` in your project should be possible in about half an hour,
while requiring less than **100** lines of code. If this is not the case for your project, please create an issue on github.

## Examples

To run the examples, download the repository and run `cargo run --example <example name>`.

- `tic-tac-toe`: A port of [Sunjay's wonderful implementation], implementing and using `game_controller` required about 40 loc.

[Stockfish]:https://www.chessprogramming.org/Stockfish
[Sunjay's wonderful implementation]: https://github.com/sunjay/tic-tac-toe.git