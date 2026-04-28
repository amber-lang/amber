use crate::utils::io::find_amber_files;
use std::path::PathBuf;

#[test]
fn test_find_amber_files_recursive() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = vec![];
    find_amber_files(
        &PathBuf::from("src/tests/io/find_amber_files/normal"),
        &mut files,
    )?;

    files.sort();
    let expected_files = vec![
        PathBuf::from("src/tests/io/find_amber_files/normal/script.ab"),
        PathBuf::from("src/tests/io/find_amber_files/normal/subdir/included.ab"),
    ];
    assert_eq!(files, expected_files);

    Ok(())
}

/// Test validating find_amber_files correctly resolving symlinks.
#[test]
fn test_find_amber_files_symlink() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = vec![];
    find_amber_files(
        &PathBuf::from("src/tests/io/find_amber_files/symlink"),
        &mut files,
    )?;

    assert_eq!(files.len(), 2);

    files.sort();
    let expected_files = vec![
        PathBuf::from("src/tests/io/find_amber_files/symlink/link/script.ab"),
        PathBuf::from("src/tests/io/find_amber_files/symlink/link/subdir/included.ab"),
    ];
    assert_eq!(files, expected_files);

    Ok(())
}

/// Test validating find_amber_files failing for looping symlinks.
#[test]
fn test_find_amber_files_loop() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = vec![];
    let result = find_amber_files(
        &PathBuf::from("src/tests/io/find_amber_files/loop"),
        &mut files,
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    // Use string to avoid unstable error kind
    assert!(err.to_string().contains("Too many levels of symbolic links"));

    Ok(())
}

