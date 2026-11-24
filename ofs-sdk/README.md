# Open Frame Studio native SDK

ABI v1 estimates the catalogue cost of one standard rectangular, fixed-glazed timber frame. It uses the same Rust frame constructor, production data and default price table as the application. Dimensions are in millimetres. Prices and labour are indicative.

The x64 Windows package contains `ofs_sdk.dll` and an MSVC import library. The x86-64 Linux package contains `libofs_sdk.so` built on Ubuntu 22.04. Include `ofs_sdk.h` and use the platform C calling convention and natural struct alignment.

`ofs_estimate_fixed_v1(width_mm, height_mm)` accepts widths of 200–6000 mm and heights of 200–4000 mm. The result contains a status code, total EUR, labour hours and the glass cost in EUR. Status 0 means success, 1 means invalid dimensions, and 2 means an internal error. All result amounts are zero on error. Arguments and results are passed by value. The caller has no buffers or handles to release. Calls share no mutable state.

The symbol and struct layout are fixed for ABI v1. A breaking change requires a new symbol. Catalogue rates can change between product versions. To reproduce a price, pin the release and platform. The Rust source entry points are `ofs_core::calculation::estimate_fixed_frame` and `estimate_cost(&Kozijn, &PriceTable)` for a configured frame. Pin the repository revision when consuming the Rust crate.

Extract the package and run the external consumer with Python 3.

```sh
python example.py ./ofs_sdk.dll
# Linux
python3 example.py ./libofs_sdk.so
```

To build from source, run `cargo build --release --locked -p ofs-sdk` and pass the library in `target/release` to `ofs-sdk/example.py`. Windows release builds sign the DLL with Authenticode before packaging.

A Rust application can consume the full frame model directly. This example uses the core from the v2026.48.4 release.

```toml
[dependencies]
ofs-core = { git = "https://github.com/OpenAEC-Foundation/open-frame-studio.git", rev = "0b2b34e3149fa298eb03f8232b4c26e9f85fc8ee", default-features = false }
```

```rust
use ofs_core::{calculation::{estimate_cost, PriceTable}, kozijn::Kozijn};

fn main() {
    let frame = Kozijn::new("Fixed frame", "K01", 1000.0, 1200.0);
    let estimate = estimate_cost(&frame, &PriceTable::default());
    println!("{:.2} EUR", estimate.total_cost);
}
```

The included repository LICENSE specifies CC BY-SA 4.0. The Rust workspace metadata specifies AGPL-3.0. The rights holders still need to reconcile these notices.
