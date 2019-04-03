# rubot

A rust library for easily usable game bots. While these controllers are most definitely far weaker and slower as a specialized engine like [Stockfish], they are good enough be an interesting challenge in most games.

Integrating `rubot` in your project should be possible in about half an hour while only requiring less than a total of **100** lines of code. 
If this is not the case for your project, please create an issue on github.

## Examples

To run the examples, download the repository and run `cargo run --example <example name>`.

- `tic-tac-toe`: A port of [Sunjay's wonderful tic-tac-toe implementation], adding and using `rubot` required about 40 loc.
- `oko`: An original game inspired by [Dots and Boxes] and tic-tac-toe.

## TODO

- documentation
- tests
- improve `alpha_beta::Bot`
- publish on `crates.io`


[Stockfish]:https://www.chessprogramming.org/Stockfish
[Sunjay's wonderful tic-tac-toe implementation]: https://github.com/sunjay/tic-tac-toe.git
[Dots and Boxes]:https://en.wikipedia.org/wiki/Dots_and_Boxes
