# Virtual Virtual Boy

## Description

Virtual Virtual Boy is an emulator for the [Virtual Boy](https://en.wikipedia.org/wiki/Virtual_Boy) 3D console. You can use it with a pair of cheap [Anaglyph 3D glasses](https://en.wikipedia.org/wiki/Anaglyph_3D) or with [Google Cardboard](https://arvr.google.com/cardboard/) to play Virtual Boy games on an Android phone.

[<img src="https://play.google.com/intl/en_us/badges/images/generic/en-play-badge.png"
     alt="Get it on Google Play"
     height="80">](https://play.google.com/store/apps/details?id=com.simongellis.vvb)

## Development

### Initial Setup
1. Install JDK8 and make sure it's in your path
2. Install [rustup](https://rustup.rs/)
3. Add targets for the four supported platforms:
```shell script
rustup target add armv7-linux-androideabi   # for arm
rustup target add i686-linux-android        # for x86
rustup target add aarch64-linux-android     # for arm64
rustup target add x86_64-linux-android      # for x86_64
```

### Building the app
```shell script
gradlew build
```

If you're testing on a specific device, you can speed up builds by setting `rust.targets` in the `local.properties` file to the architecture you care about.
```properties
# only compile for x86_64
rust.targets=x86_64
```

### Adding new bundled games
1. Add a .vb file to [./app/src/main/assets/games].
2. Add an entry to [./app/src/main/res/raw/bundledgames.json].

### Running tests
```shell script
cargo test
```

## Known Issues

Several features are not implemented:
 - Some VIP interrupts (TIMEERR, SBHIT)
 - Game Pad and Game Pak interrupts
 - Game Pak expansions and the link cable
 - The instruction cache

## Credits
- Guy Perfect for writing the extremely helpful [Virtual Boy Sacred Tech Scroll](https://virtual-boy.com/documents/virtual-boy-sacred-tech-scroll)
- [Pi Lanningham](https://github.com/Quantumplation) and [Robert Kellett](https://github.com/Splagoon) for rubber ducking
- [JakeDowns](https://github.com/jakedowns) for adding support for Leia 3D devices
- [AngelofMe](https://github.com/AngelofMe) and [soundsnow](https://github.com/soundsnow) for the Chinese, Dutch, Finnish, French, German, Greek, Hungarian, Indonesian, Italian, Japanese, Korean, Lithuanian, Polish, Portuguese, Russian, Spanish, Swedish, Turkish translations
