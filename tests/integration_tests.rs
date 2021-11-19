mod util;
use util::*;

#[test]
fn a_argparse() -> std::io::Result<()> {
    // Check that the arg parsing works
    run_command(vec!["help"])
}

#[test]
fn b_add_mod() -> std::io::Result<()> {
    // Add Sodium to config
    run_command(vec!["add", "sodium"])?;
    // Check that trying to add the same mod again fails
    assert!(run_command(vec!["add", "SoDiUm"]).is_err());

    Ok(())
}

#[test]
fn c_add_repo() -> std::io::Result<()> {
    // Add Sodium to config
    run_command(vec!["add-repo", "CaffeineMC", "sodium-fabric"])?;
    // Check that trying to add the same repo again fails
    assert!(run_command(vec!["add-repo", "caffeinemc", "Sodium-Fabric"]).is_err());

    Ok(())
}

#[test]
fn d_list() -> std::io::Result<()> {
    run_command(vec!["list"])
}

#[test]
fn e_list_verbose() -> std::io::Result<()> {
    run_command(vec!["list", "--verbose"])
}

#[test]
fn f_remove() -> std::io::Result<()> {
    // Make test runner remove all mods. This is why this test is semi-automatic
    run_command_visible(vec!["remove"])?;
    // Check that listing mods gives an error (no mods/repos in config)
    assert!(run_command(vec!["list"]).is_err());

    Ok(())
}
