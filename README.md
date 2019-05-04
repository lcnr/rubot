# rubot

A rust library for easily usable game bots. While these controllers are a lot weaker and slower than a specialized engine like [Stockfish], they are good enough be a challenge in most games.

Integrating `rubot` in your project should be possible in about half an hour while only requiring less than a total of **100** lines of code. 
If this is not the case for your project, please create an issue on github.

## Examples

To run the examples, download the repository and run `cargo run --example <example name>`.

- `tic-tac-toe`: A port of [Sunjay's wonderful tic-tac-toe implementation][sunjay], adding and using `rubot` required about 40 loc.
- `chess`: A chess bot using [shakmaty].
- `oko`: An original game inspired by [Dots and Boxes] and tic-tac-toe.

## Supported games

`rubot` only supports deterministic games with perfect information.

[shakmaty]:https://crates.io/crates/shakmaty
[Stockfish]:https://www.chessprogramming.org/Stockfish
[sunjay]: https://github.com/sunjay/tic-tac-toe.git
[Dots and Boxes]:https://en.wikipedia.org/wiki/Dots_and_Boxes
[fow]: https://en.wikipedia.org/wiki/Fog_of_war#In_video_games
