#[macro_use] extern crate enum_primitive;

mod gif {
    enum_from_primitive! {
    /// Known block types
    enum Block {
        Image = 0x2C,
        Extension = 0x21,
        Trailer = 0x3B
    }
    }

    enum_from_primitive! {
    /// Known GIF extensions
    enum Extension {
        Text = 0x01,
        Control = 0xF9,
        Comment = 0xFE,
        Application = 0xFF
    }
    }

    enum_from_primitive! {
    /// Method to dispose the image
    enum DisposalMethod {
        Undefined = 0,
        None = 1,
        Previous = 2,
        Background = 3
    }
    }
}

mod png {
    enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum InterlaceMethod {
        None = 0,
        Adam7 = 1
    }
    }

    enum_from_primitive! {
    #[derive(Debug)]
    pub enum FilterType {
        NoFilter = 0,
        Sub = 1,
        Up = 2,
        Avg = 3,
        Paeth = 4
    }
    }
}

mod tiff {
    enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum PhotometricInterpretation {
        WhiteIsZero = 0,
        BlackIsZero = 1,
        RGB = 2,
        RGBPalette = 3,
        TransparencyMask = 4,
        CMYK = 5,
        YCbCr = 6,
        CIELab = 8,
    }
    }

    enum_from_primitive! {
    #[derive(Clone, Copy, Debug)]
    enum CompressionMethod {
        None = 1,
        Huffman = 2,
        Fax3 = 3,
        Fax4 = 4,
        LZW = 5,
        JPEG = 6,
        PackBits = 32773
    }
    }

    enum_from_primitive! {
    #[derive(Clone, Copy, Debug)]
    enum PlanarConfiguration {
        Chunky = 1,
        Planar = 2
    }
    }

    enum_from_primitive! {
    #[derive(Clone, Copy, Debug)]
    enum Predictor {
        None = 1,
        Horizontal = 2
    }
    }

    enum_from_primitive! {
    #[derive(Clone, Copy, Debug)]
    pub enum Type {
        BYTE = 1,
        ASCII = 2,
        SHORT = 3,
        LONG = 4,
        RATIONAL = 5,
    }
    }
}
