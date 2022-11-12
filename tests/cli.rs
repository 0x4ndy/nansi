use std::error::Error;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("nansi")?;

    cmd.arg("test/file/doesnt/exist");
    cmd.assert().failure().stderr(predicate::str::contains("No such file or directory"));

    Ok(())
}

#[test]
fn linux_file() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("nansi")?;

    cmd.arg("testdata/nansifile_linux.json");

    let output = "Using NansiFile: testdata/nansifile_linux.json\n[\u{1b}[38;5;10mOK\u{1b}[39m] [1][ls] ls \n[\u{1b}[38;5;9mFAIL\u{1b}[39m] [2][l2] ls -12345\n[\u{1b}[38;5;9mFAIL\u{1b}[39m] [3][asd] aaa \nNo such file or directory (os error 2)\n[\u{1b}[38;5;10mOK\u{1b}[39m] [4][bash] /bin/bash -c ls -ltra | grep README\n";
    
    cmd.assert().success().stdout(predicate::str::contains(output.to_string()));

    Ok(())
}

#[test]
fn linux_duplicate_labels_file() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("nansi")?;

    cmd.arg("testdata/nansifile_linux_duplicate_labels.json");

    let output = "Using NansiFile: testdata/nansifile_linux_duplicate_labels.json\n\u{1b}[38;5;11m[WARN]\u{1b}[39m The following aliases are duplicated which may cause issues with conditional execution:\n[\"asd\", \"ls\"]\n[\u{1b}[38;5;10mOK\u{1b}[39m] [1][ls] ls \n[\u{1b}[38;5;9mFAIL\u{1b}[39m] [2] ls -12345\n[\u{1b}[38;5;9mFAIL\u{1b}[39m] [3][asd] aaa \nNo such file or directory (os error 2)\n[\u{1b}[38;5;10mOK\u{1b}[39m] [4][ls] ls \n[\u{1b}[38;5;9mFAIL\u{1b}[39m] [5][asd] aaa \nNo such file or directory (os error 2)\n[\u{1b}[38;5;10mOK\u{1b}[39m] [6] /bin/bash -c ls -ltra | grep README\n";

    cmd.assert().success().stdout(predicate::str::contains(output.to_string()));

    Ok(())
}

#[test]
fn linux_prereq_file() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("nansi")?;

    cmd.arg("testdata/nansifile_linux_prereq.json");

    let output = "Using NansiFile: testdata/nansifile_linux_prereq.json\n[\u{1b}[38;5;10mOK\u{1b}[39m] [1][ls] ls \n[\u{1b}[38;5;3mSKIP\u{1b}[39m] [2][lsls] ls \nPrerequisites for item [1][lsls] are not met.\n[\u{1b}[38;5;9mFAIL\u{1b}[39m] [3][l2] ls -12345\n[\u{1b}[38;5;9mFAIL\u{1b}[39m] [4][asd] aaa \nNo such file or directory (os error 2)\n[\u{1b}[38;5;3mSKIP\u{1b}[39m] [5][bash] /bin/bash -c ls -ltra | grep README\nPrerequisites for item [4][bash] are not met.\n[\u{1b}[38;5;10mOK\u{1b}[39m] [6] ls \n";

    cmd.assert().success().stdout(predicate::str::contains(output.to_string()));

    Ok(())
}
