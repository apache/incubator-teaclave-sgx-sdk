#![cfg_attr(feature="nightly", feature(const_fn))]
#![doc(html_root_url = "https://docs.rs/yansi/0.3")]

//! A dead simple ANSI terminal color painting library.
//!
//! # Usage
//!
//! Usage is best illustrated via a quick example:
//!
//! ```rust
//! use yansi::Paint;
//! use yansi::Color::White;
//!
//! println!("Testing, {}, {}, {}!", Paint::red(1),
//!     Paint::green(2).bold().underline(),
//!     Paint::blue("3").bg(White).italic());
//! ```
//!
//! ## `Paint`
//!
//! The main entry point into this library is the `Paint` type. `Paint`
//! encapsulates a value of any type that implements the `Display` or `Debug`
//! trait. When a `Paint` is `Display`ed or `Debug`ed, the appropriate ANSI
//! escape characters are emitted before and after the wrapped type's `fmt`
//! implementation.
//!
//! `Paint` can be constructed via any of following methods: [`black`], [`red`],
//! [`green`], [`yellow`], [`blue`], [`purple`], [`cyan`], [`white`]. Each of
//! these methods sets the foreground color of the item to be displayed
//! according to the name of the method. Additionally, [`rgb`] and [`fixed`]
//! allow you to customize the foreground color to your liking. In addition to
//! these constructors, you can also use the [`paint`] method on a given
//! [`Color`] value to construct a `Paint` type. Both of these approaches are
//! shown below:
//!
//! ```rust
//! use yansi::Paint;
//! use yansi::Color::Red;
//!
//! println!("I'm {}!", Paint::red("red").bold());
//! println!("I'm also {}!", Red.paint("red").bold());
//! ```
//!
//! [`black`]: struct.Paint.html#method.black,
//! [`red`]: struct.Paint.html#method.red,
//! [`green`]: struct.Paint.html#method.green,
//! [`yellow`]: struct.Paint.html#method.yellow,
//! [`blue`]: struct.Paint.html#method.blue,
//! [`purple`]: struct.Paint.html#method.purple,
//! [`cyan`]: struct.Paint.html#method.cyan,
//! [`white`]: struct.Paint.html#method.white
//! [`paint`]: enum.Color.html#method.paint
//! [`Color`]: enum.Color.html
//! [`rgb`]: struct.Paint.html#method.rgb
//! [`fixed`]: struct.Paint.html#method.fixed
//!
//! Finally, [`new`](struct.Paint.html#method.new) creates a `Paint` item
//! _without_ a foreground color applied.
//!
//! ## Styling
//!
//! Modifications to the styling of the item can be added via the followiing
//! chainable builder methods: [`fg`], [`bg`], [`bold`], [`dimmed`], [`italic`],
//! [`underline`], [`blink`], [`invert`], [`hidden`], [`strikethrough`].
//!
//! [`fg`]: struct.Paint.html#method.fg
//! [`bg`]: struct.Paint.html#method.bg
//! [`bold`]: struct.Paint.html#method.bold
//! [`dimmed`]: struct.Paint.html#method.dimmed
//! [`italic`]: struct.Paint.html#method.italic
//! [`underline`]: struct.Paint.html#method.underline
//! [`blink`]: struct.Paint.html#method.blink
//! [`invert`]: struct.Paint.html#method.invert
//! [`hidden`]: struct.Paint.html#method.hidden
//! [`strikethrough`]: struct.Paint.html#method.strikethrough
//!
//! # Disabling
//!
//! On Rust nightly and with the `nightly` feature enabled, painting can be
//! disabled globally via the [`Paint::disable()`] method. When painting is
//! disabled, the `Display` and `Debug` implementations for `Paint` will emit
//! the `Display` or `Debug` of the contained object and nothing else. Painting
//! can be reenabled via the [`Paint::enable()`] method.
//!
//! One potential use of this feature is to allow users to control color ouput
//! via an environment variable. For instance, to disable coloring if the
//! `CLICOLOR` variable is set to `0`, you might write:
//!
//! ```rust
//! # #[cfg(feature = "nightly")]
//! # { if false { // we don't actually want to disable coloring
//! use yansi::Paint;
//!
//! if let Ok(true) = std::env::var("CLICOLOR").map(|v| v == "0") {
//!     Paint::disable();
//! }
//! # } }
//! ```
//!
//! [`Paint::disable()`]: struct.Paint.html#method.disable
//! [`Paint::enable()`]: struct.Paint.html#method.disable
//!
//! ## Masking
//!
//! `Paint` structures can _mask_ arbitrary values. When a value is masked and
//! painting is disabled, the `Display` and `Debug` implementations of `Paint`
//! write nothing. This allows you to selectively omit output when painting is
//! disabled. Values can be masked using the [`mask`] builder method or
//! [`Paint::masked()`] constructor.
//!
//! [`mask`]: struct.Paint.html#method.mask
//! [`Paint::masked()`]: struct.Paint.html#method.masked
//!
//! One use for this feature is to print certain characters only when painting
//! is enabled. For instance, you might wish to emit the 🎨 emoji when
//! coloring is enabled but not otherwise. This can be accomplished by masking
//! the emoji:
//!
//! ```rust
//! use yansi::Paint;
//!
//! println!("I like colors!{}", Paint::masked(" 🎨"));
//! ```
//!
//! This will print "I like colors! 🎨" when painting is enabled and "I like
//! colors!" when painting is disabled.
//!
//! # Windows
//!
//! Since the Windows 10 anniversary update, Windows consoles support ANSI
//! escape sequences. This support, however, must be explicitly enabled. `yansi`
//! provides the [`Paint::enable_windows_ascii()`] method to enable ASCII
//! support on Windows consoles when available.
//!
//! ```rust
//! use yansi::Paint;
//!
//! // Enable ASCII escape sequence support on Windows consoles.
//! Paint::enable_windows_ascii();
//! ```
//!
//! [`Paint::enable_windows_ascii()`]: struct.Paint.html#method.enable_windows_ascii
//!
//! # Why?
//!
//! Several terminal coloring libraries exist ([`ansi_term`], [`colored`],
//! [`term_painter`], to name a few), begging the question: why yet another?
//! Here are a few reasons:
//!
//!   * This library is _much_ simpler: there are two types! The complete
//!     implementation is only about 250 lines of code.
//!   * Like [`term_painter`], but unlike [`ansi_term`], _any_ type implementing
//!     `Display` can be stylized, not only strings.
//!   * Styling can be enabled and disabled on the fly.
//!   * Arbitrary items can be _masked_ for selective disabling.
//!   * Typically, only one type needs to be imported: `Paint`.
//!   * Zero dependencies. It really is simple.
//!   * The name `yansi` is pretty short.
//!
//! All that being said, this library borrows the general API from the three
//! libraries as well as plenty of code from [`ansi_term`].
//!
//! [`ansi_term`]: https://crates.io/crates/ansi_term
//! [`colored`]: https://crates.io/crates/colored
//! [`term_painter`]: https://crates.io/crates/term-painter

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
extern crate sgx_tstd as std;

use std::fmt::{self, Display};

#[cfg(test)] mod tests;
mod windows;

#[inline(always)]
fn write_spliced<T: Display>(c: &mut bool, f: &mut fmt::Formatter, t: T) -> fmt::Result {
    if *c {
        write!(f, ";{}", t)
    } else {
        *c = true;
        write!(f, "{}", t)
    }
}

/// An enum representing an ANSI color code.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
pub enum Color {
    /// No color has been set. Nothing is changed when applied.
    Unset,

    /// Black #0 (foreground code `30`, background code `40`).
    Black,

    /// Red: #1 (foreground code `31`, background code `41`).
    Red,

    /// Green: #2 (foreground code `32`, background code `42`).
    Green,

    /// Yellow: #3 (foreground code `33`, background code `43`).
    Yellow,

    /// Blue: #4 (foreground code `34`, background code `44`).
    Blue,

    /// Purple: #5 (foreground code `35`, background code `45`).
    Purple,

    /// Cyan: #6 (foreground code `36`, background code `46`).
    Cyan,

    /// White: #7 (foreground code `37`, background code `47`).
    White,

    /// A color number from 0 to 255, for use in 256-color terminals.
    Fixed(u8),

    /// A 24-bit RGB color, as specified by ISO-8613-3.
    RGB(u8, u8, u8),
}

impl Color {
    /// Constructs a new `Paint` structure that encapsulates `item` with the
    /// foreground color set to the color `self`.
    ///
    /// ```rust
    /// use yansi::Color::Blue;
    ///
    /// println!("This is going to be blue: {}", Blue.paint("yay!"));
    /// ```
    #[inline(always)]
    pub fn paint<T>(self, item: T) -> Paint<T> {
        Paint::new(item).fg(self)
    }
}

#[doc(hidden)]
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Color::Unset => Ok(()),
            Color::Black => write!(f, "0"),
            Color::Red => write!(f, "1"),
            Color::Green => write!(f, "2"),
            Color::Yellow => write!(f, "3"),
            Color::Blue => write!(f, "4"),
            Color::Purple => write!(f, "5"),
            Color::Cyan => write!(f, "6"),
            Color::White => write!(f, "7"),
            Color::Fixed(num) => write!(f, "8;5;{}", num),
            Color::RGB(r, g, b) => write!(f, "8;2;{};{};{}", r, g, b)
        }
    }
}

impl Default for Color {
    #[inline(always)]
    fn default() -> Self { Color::Unset }
}

#[repr(packed)]
#[derive(Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
struct Style {
    bold: bool,
    dimmed: bool,
    italic: bool,
    underline: bool,
    blink: bool,
    invert: bool,
    hidden: bool,
    strikethrough: bool,
}

/// A structure encapsulating all of the styling for a given item.
///
/// See the [crate level documentation](./) for usage information.
#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
pub struct Paint<T> {
    item: T,
    foreground: Color,
    background: Color,
    style: Style,
    masked: bool,
}

macro_rules! constructors_for {
    ($T:ty, $($name:ident: $color:ident),*) => ($(
        /// Constructs a new `Paint` structure that encapsulates `item` with the
        /// foreground color set to the name of this method.
        ///
        /// ```rust
        /// use yansi::Paint;
        ///
        /// println!("This is going to be blue: {}", Paint::blue("yay!"));
        /// ```
        pub fn $name(item: $T) -> Paint<$T> {
            Paint::new(item).fg(Color::$color)
        }
    )*)
}

macro_rules! style_builder_for {
    ($T:ty, $($name:ident),*) => ($(
        /// Enables the styling corresponding to the name of this method.
        ///
        /// ```rust
        /// use yansi::Paint;
        ///
        /// println!("Red, underlined: {}", Paint::red("beep.").underline());
        /// ```
        #[inline(always)]
        pub fn $name(mut self) -> Paint<$T> {
            self.style.$name = true;
            self
        }
    )*)
}

impl<T> Paint<T> {
    /// Constructs a new `Paint` structure that encapsulates `item`. No styling
    /// is applied.
    ///
    /// ```rust
    /// use yansi::Paint;
    ///
    /// assert_eq!(Paint::new("hello!").to_string(), "hello!".to_string());
    /// ```
    #[inline(always)]
    pub fn new(item: T) -> Paint<T> {
        Paint {
            item: item,
            foreground: Color::default(),
            background: Color::default(),
            style: Style::default(),
            masked: false
        }
    }

    /// Constructs a new `Paint` structure that encapsulates `item` and masks
    /// it. No styling is applied. A masked item is not written out when
    /// painting is disabled during `Display` or `Debug` invocations. When
    /// painting is enabled, masking has no effect.
    ///
    /// ```rust
    /// use yansi::Paint;
    ///
    /// println!("{}Sprout!", Paint::masked("🌱 "));
    /// ```
    #[inline(always)]
    pub fn masked(item: T) -> Paint<T> {
        Paint::new(item).mask()
    }

    constructors_for!(T, black: Black, red: Red, green: Green, yellow: Yellow,
                         blue: Blue, purple: Purple, cyan: Cyan, white: White);

    /// Constructs a new `Paint` structure that encapsulates `item` with the
    /// foreground color set RGB color corresponding to `r`, `g`, `b`.
    ///
    /// ```rust
    /// use yansi::Paint;
    ///
    /// println!("This is going to be funky: {}", Paint::rgb(70, 130, 122, "hi!"));
    /// ```
    #[inline(always)]
    pub fn rgb(r: u8, g: u8, b: u8, item: T) -> Paint<T> {
        Paint::new(item).fg(Color::RGB(r, g, b))
    }

    /// Constructs a new `Paint` structure that encapsulates `item` with the
    /// foreground color set to the fixed color corresponding to `color`.
    ///
    /// ```rust
    /// use yansi::Paint;
    ///
    /// println!("This is going to be funky: {}", Paint::fixed(100, "hi!"));
    /// ```
    #[inline(always)]
    pub fn fixed(color: u8, item: T) -> Paint<T> {
        Paint::new(item).fg(Color::Fixed(color))
    }

    /// Sets the foreground to `color`.
    ///
    /// ```rust
    /// use yansi::Paint;
    /// use yansi::Color::Red;
    ///
    /// println!("Red foreground: {}", Paint::new("hi!").fg(Red));
    /// ```
    #[inline(always)]
    pub fn fg(mut self, color: Color) -> Paint<T> {
        self.foreground = color;
        self
    }

    /// Sets the background to `color`.
    ///
    /// ```rust
    /// use yansi::Paint;
    /// use yansi::Color::Yellow;
    ///
    /// println!("Yellow background: {}", Paint::new("hi!").bg(Yellow));
    /// ```
    #[inline(always)]
    pub fn bg(mut self, color: Color) -> Paint<T> {
        self.background = color;
        self
    }

    /// Masks `self`. A masked item is not written out when painting is disabled
    /// during `Display` or `Debug` invocations. When painting is enabled,
    /// masking has no effect.
    ///
    /// ```rust
    /// use yansi::Paint;
    ///
    /// println!("{}Something happened.", Paint::red("Whoops! ").mask());
    /// ```
    #[inline(always)]
    pub fn mask(mut self) -> Paint<T> {
        self.masked = true;
        self
    }

    style_builder_for!(T, bold, dimmed, italic, underline, blink, invert, hidden, strikethrough);

    #[inline]
    fn is_plain(&self) -> bool {
        self.foreground == Color::default()
            && self.background == Color::default()
            && self.style == Style::default()
    }

    fn styles(&self) -> [bool; 10] {
        [false, self.style.bold, self.style.dimmed, self.style.italic, self.style.underline,
            self.style.blink, false, self.style.invert, self.style.hidden, self.style.strikethrough]
    }

    /// Write any ANSI codes that go *before* a piece of text. These should be
    /// the codes to set the terminal to a different colour or font style.
    fn write_prefix(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // A user may just want a code-free string when no styles are applied.
        if self.is_plain() {
            return Ok(());
        }

        let mut splice = false;
        write!(f, "\x1B[")?;

        for (i, _) in self.styles().iter().enumerate().filter(|&(_, e)| *e) {
            write_spliced(&mut splice, f, i)?;
        }

        if self.background != Color::Unset {
            write_spliced(&mut splice, f, "4")?;
            self.background.fmt(f)?;
        }

        if self.foreground != Color::Unset {
            write_spliced(&mut splice, f, "3")?;
            self.foreground.fmt(f)?;
        }

        // All the codes end with an `m`, because reasons.
        write!(f, "m")
    }

    /// Write any ANSI codes that go *after* a piece of text. These should be
    /// the codes to *reset* the terminal back to its normal colour and style.
    fn write_suffix(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_plain() {
            return Ok(());
        }

        write!(f, "\x1B[0m")
    }
}

#[cfg(feature="nightly")] use std::sync::atomic::AtomicBool;
#[cfg(feature="nightly")] use std::sync::atomic::Ordering;
#[cfg(feature="nightly")] static DISABLED: AtomicBool = AtomicBool::new(false);

impl Paint<()> {
    /// Disables coloring globally.
    ///
    /// This method is only available when the "nightly" feature is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use yansi::Paint;
    ///
    /// // With coloring enabled, ANSI color codes are emitted.
    /// assert_ne!(Paint::green("go").to_string(), "go".to_string());
    ///
    /// // With coloring disabled, ANSI color codes are _not_ emitted.
    /// Paint::disable();
    /// assert_eq!(Paint::green("go").to_string(), "go".to_string());
    /// ```
    #[cfg(feature="nightly")]
    pub fn disable() {
        DISABLED.store(true, Ordering::Release);
    }

    /// Enables coloring globally. Coloring is enabled by default, so this
    /// method should only be called to _re_ enable coloring.
    ///
    /// This method is only available when the "nightly" feature is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use yansi::Paint;
    ///
    /// // With coloring disabled, ANSI color codes are _not_ emitted.
    /// Paint::disable();
    /// assert_eq!(Paint::green("go").to_string(), "go".to_string());
    ///
    /// // Reenabling causes color code to be emitted.
    /// Paint::enable();
    /// assert_ne!(Paint::green("go").to_string(), "go".to_string());
    /// ```
    #[cfg(feature="nightly")]
    pub fn enable() {
        DISABLED.store(false, Ordering::Release);
    }

    /// Enables ASCII terminal escape sequences on Windows consoles when
    /// possible. Returns `true` if escape sequence support was successfully
    /// enabled and `false` otherwise. On non-Windows targets, this method
    /// always returns `true`.
    ///
    /// Support for escape sequences in Windows consoles was added in the
    /// Windows 10 anniversary update. For targets with older Windows
    /// installations, this method is expected to return `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use yansi::Paint;
    ///
    /// // A best-effort Windows ASCII terminal support enabling.
    /// Paint::enable_windows_ascii();
    /// ```
    #[inline(always)]
    pub fn enable_windows_ascii() -> bool {
        windows::enable_ascii_colors()
    }
}

fn paint_enabled() -> bool {
    #[cfg(feature="nightly")]
    { !DISABLED.load(Ordering::Relaxed) }

    #[cfg(not(feature="nightly"))]
    { true }
}

impl<T: fmt::Display> fmt::Display for Paint<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if paint_enabled() {
            self.write_prefix(f)?;
            self.item.fmt(f)?;
            self.write_suffix(f)
        } else if !self.masked {
            self.item.fmt(f)
        } else {
            Ok(())
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Paint<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if paint_enabled() {
            self.write_prefix(f)?;
            self.item.fmt(f)?;
            self.write_suffix(f)
        } else if !self.masked {
            self.item.fmt(f)
        } else {
            Ok(())
        }
    }
}
