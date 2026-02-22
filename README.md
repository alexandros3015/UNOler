# UNOler

An UNO-style card game written in Rust.

## Usage

To play the game, simply run the following command in your terminal:

```bash
cargo run
```

Or, you can download the binary from the releases page.
> Note: The binary is statically linked, so it will not work on non-Windows or non-x86_64 architectures.

## Features

- Supports multiple players
- Supports color choices
- Supports plus fours and twos
- Supports reverses
- Supports custom OS-specific randomization

## Limitations

- Only supports Windows and Linux
- Only supports x86_64 and ARM64 architectures
- Only supports Windows 10 and newer

## Contributing

Ideas are welcome. If you have any suggestions or improvements, open an issue.   
Please do not open a pull request without discussing the change first.  
Or you could just take it and make it yourself.

## How to play

1. Enter the number of players you want to play with.
2. Discard a card from your hand if it is a legal play.
3. Use "d" or "draw" to draw a card or "s" or "see" to see your hand.
4. The first player to run out of cards wins.

## Rules

See the full game rules here:  
[RULES.md](RULES.md)

## License

This project is released under the MIT License. See the LICENSE file for more information.

### Disclaimer

This project is an independent, fan-made implementation inspired by the mechanics of [UNO](https://en.wikipedia.org/wiki/UNO_(card_game)).
This project is not affiliated with or endorsed by Mattel.
