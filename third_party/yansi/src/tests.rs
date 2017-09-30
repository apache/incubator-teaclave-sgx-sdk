use super::Color::*;
use super::Paint;

macro_rules! test {
    ($name:ident: $input:expr => $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!($input.to_string(), $expected.to_string());
        }
    };
}

test!(plain: Paint::new("text/plain") => "text/plain");
test!(red: Paint::red("hi") => "\x1B[31mhi\x1B[0m");
test!(black: Paint::black("hi") => "\x1B[30mhi\x1B[0m");
test!(yellow_bold: Paint::yellow("hi").bold() => "\x1B[1;33mhi\x1B[0m");
test!(yellow_bold_2: Paint::new("hi").fg(Yellow).bold() => "\x1B[1;33mhi\x1B[0m");
test!(blue_underline: Paint::blue("hi").underline() => "\x1B[4;34mhi\x1B[0m");
test!(green_bold_ul: Paint::green("hi").bold().underline() => "\x1B[1;4;32mhi\x1B[0m");
test!(green_bold_ul_2: Paint::green("hi").underline().bold() => "\x1B[1;4;32mhi\x1B[0m");
test!(purple_on_white: Paint::purple("hi").bg(White) => "\x1B[47;35mhi\x1B[0m");
test!(yellow_on_blue: Paint::red("hi").bg(Blue).fg(Yellow) => "\x1B[44;33mhi\x1B[0m");
test!(yellow_on_blue_2: Paint::cyan("hi").bg(Blue).fg(Yellow) => "\x1B[44;33mhi\x1B[0m");
test!(cyan_bold_on_white: Paint::cyan("hi").bold().bg(White) => "\x1B[1;47;36mhi\x1B[0m");
test!(cyan_ul_on_white: Paint::cyan("hi").underline().bg(White) => "\x1B[4;47;36mhi\x1B[0m");
test!(cyan_bold_ul_on_white: Paint::cyan("hi").bold().underline().bg(White)
      => "\x1B[1;4;47;36mhi\x1B[0m");
test!(cyan_ul_bold_on_white: Paint::cyan("hi").underline().bold().bg(White)
      => "\x1B[1;4;47;36mhi\x1B[0m");
test!(fixed: Paint::fixed(100, "hi") => "\x1B[38;5;100mhi\x1B[0m");
test!(fixed_on_purple: Paint::fixed(100, "hi").bg(Purple) => "\x1B[45;38;5;100mhi\x1B[0m");
test!(fixed_on_fixed: Paint::fixed(100, "hi").bg(Fixed(200)) => "\x1B[48;5;200;38;5;100mhi\x1B[0m");
test!(rgb: Paint::rgb(70, 130, 180, "hi") => "\x1B[38;2;70;130;180mhi\x1B[0m");
test!(rgb_on_blue: Paint::rgb(70, 130, 180, "hi").bg(Blue) => "\x1B[44;38;2;70;130;180mhi\x1B[0m");
test!(blue_on_rgb: Paint::blue("hi").bg(RGB(70, 130, 180)) => "\x1B[48;2;70;130;180;34mhi\x1B[0m");
test!(rgb_on_rgb: Paint::rgb(70, 130, 180, "hi").bg(RGB(5,10,15))
      => "\x1B[48;2;5;10;15;38;2;70;130;180mhi\x1B[0m");
test!(bold: Paint::new("hi").bold() => "\x1B[1mhi\x1B[0m");
test!(underline: Paint::new("hi").underline() => "\x1B[4mhi\x1B[0m");
test!(bunderline: Paint::new("hi").bold().underline() => "\x1B[1;4mhi\x1B[0m");
test!(dimmed: Paint::new("hi").dimmed() => "\x1B[2mhi\x1B[0m");
test!(italic: Paint::new("hi").italic() => "\x1B[3mhi\x1B[0m");
test!(blink: Paint::new("hi").blink() => "\x1B[5mhi\x1B[0m");
test!(invert: Paint::new("hi").invert() => "\x1B[7mhi\x1B[0m");
test!(hidden: Paint::new("hi").hidden() => "\x1B[8mhi\x1B[0m");
test!(stricken: Paint::new("hi").strikethrough() => "\x1B[9mhi\x1B[0m");
