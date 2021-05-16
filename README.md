# Virtual Virtual Boy

## Description

Virtual Virtual Boy is an emulator for the [Virtual Boy](https://en.wikipedia.org/wiki/Virtual_Boy) 3D console. You can use it with a pair of cheap [Anaglyph 3D glasses](https://en.wikipedia.org/wiki/Anaglyph_3D) or with [Google Cardboard](https://arvr.google.com/cardboard/) to play Virtual Boy games on an Android phone.

## Development
Build the app:
```shell script
gradlew build
```
Run tests:
```shell script
cargo test
```

## Known Issues

Several features are not implemented:
 - Software Game Pad reads (every game I tested against used hardware reads)
 - Some VIP interrupts (TIMEERR, SBHIT)
 - Game Pad and Game Pak interrupts
 - Game Pak expansions and the link cable
 - The instruction cache

## Credits
- Guy Perfect for writing the extremely helpful [Virtual Boy Sacred Tech Scroll](https://www.virtual-boy.com/documents/virtual-boy-sacred-tech-scroll/)
- [Pi Lanningham](https://github.com/Quantumplation) and [Robert Kellett](https://github.com/Splagoon) for rubber ducking