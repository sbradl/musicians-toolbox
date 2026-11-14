fn read() -> u8 {
    let Some(byte) = first() else {
        return 0; // coverage: only a broken file reaches this
    };

    if byte > 9 {
        return 9;
    }

    byte
}
