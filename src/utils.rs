macro_rules! shake256 {
    ( $output:expr ; $( $input:expr ),* ) => {
        let mut hasher = ::sha3::Shake256::default();
        $(
            ::digest::Update::update(&mut hasher, &$input[..]);
        )*
        let mut reader = ::digest::ExtendableOutput::finalize_xof(hasher);
        ::digest::XofReader::read(&mut reader, $output);
    }
}

macro_rules! sha3_256 {
    ( $output:expr ; $( $input:expr ),* ) => {
        let mut hasher = ::sha3::Sha3_256::default();
        $(
            ::digest::Update::update(&mut hasher, &$input[..]);
        )*
        $output.copy_from_slice(::digest::FixedOutput::finalize_fixed(hasher).as_slice());
    }
}

macro_rules! sha3_512 {
    ( $output:expr ; $( $input:expr ),* ) => {
        let mut hasher = ::sha3::Sha3_512::default();
        $(
            ::digest::Update::update(&mut hasher, &$input[..]);
        )*
        $output.copy_from_slice(::digest::FixedOutput::finalize_fixed(hasher).as_slice());
    }
}
