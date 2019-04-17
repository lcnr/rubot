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

While `rubot` tries to be usable with as many different kinds of games as possible, there are some limitations
which may or may not be lifted in the future.

`rubot` currently requires the game to be deterministic. This prevents games where the player is missing information, like *Rock Paper Scissors* or [Durak].

## TODO

- publish on `crates.io`
- allow for non deterministic games

[Durak]:https://en.wikipedia.org/wiki/Durak
[shakmaty]:https://crates.io/crates/shakmaty
[Stockfish]:https://www.chessprogramming.org/Stockfish
[sunjay]: https://github.com/sunjay/tic-tac-toe.git
[Dots and Boxes]:https://en.wikipedia.org/wiki/Dots_and_Boxes
[fow]: https://en.wikipedia.org/wiki/Fog_of_war#In_video_games
