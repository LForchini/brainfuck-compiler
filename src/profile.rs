use crate::Token;
use once_cell::sync::Lazy;
use platform_dirs::AppDirs;
use serde::Deserialize;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

static CONFIG_PATH: Lazy<PathBuf> =
    Lazy::new(|| AppDirs::new(Some("bfc"), true).unwrap().config_dir);
static CACHE_PATH: Lazy<PathBuf> = Lazy::new(|| AppDirs::new(Some("bfc"), true).unwrap().cache_dir);

static PROFILES: Lazy<Vec<Profile>> = Lazy::new(|| {
    let mut profiles = vec![];

    for entry in fs::read_dir(CONFIG_PATH.as_path()).unwrap().flatten() {
        if entry.file_type().unwrap().is_file() {
            if let Ok(s) = fs::read_to_string(entry.path()) {
                let str = Box::leak(s.into_boxed_str());

                if let Ok(profile) = serde_json::from_str::<Profile>(str) {
                    profiles.push(profile);
                }
            }
        }
    }

    profiles
});
static DEFAULT_PROFILE: Lazy<&str> = Lazy::new(|| {
    if std::env::consts::OS == "macos" {
        "macos_64"
    } else {
        "elf_32"
    }
});

#[derive(Clone, Debug, Deserialize)]
pub struct Profile {
    name: &'static str,

    setup: Vec<&'static str>,
    teardown: Vec<&'static str>,

    ptradd: Vec<&'static str>,
    ptrsub: Vec<&'static str>,
    add: Vec<&'static str>,
    sub: Vec<&'static str>,
    loopstart: Vec<&'static str>,
    loopend: Vec<&'static str>,
    putchar: Vec<&'static str>,
    getchar: Vec<&'static str>,

    nasm_args: Vec<&'static str>,
    linker: &'static str,
    linker_args: Vec<&'static str>,
}

impl Profile {
    pub fn get_setup_asm(&self) -> String {
        self.setup.join("\n")
    }

    pub fn get_teardown_asm(&self) -> String {
        self.teardown.join("\n")
    }

    pub fn get_asm(&self, tok: Token) -> String {
        match tok {
            Token::PtrAdd(n) => self.ptradd.join("\n").replace("{}", &n.to_string()),
            Token::PtrSub(n) => self.ptrsub.join("\n").replace("{}", &n.to_string()),
            Token::Add(n) => self.add.join("\n").replace("{}", &n.to_string()),
            Token::Sub(n) => self.sub.join("\n").replace("{}", &n.to_string()),
            Token::LoopStart(n) => self.loopstart.join("\n").replace("{}", &n.to_string()),
            Token::LoopEnd(n) => self.loopend.join("\n").replace("{}", &n.to_string()),
            Token::PutChar => self.putchar.join("\n"),
            Token::GetChar => self.getchar.join("\n"),
        }
    }

    pub fn generate_bin(&self, asm: &[String], outfile: &Path) -> Result<(), io::Error> {
        let mut asm_path = CACHE_PATH.clone();
        asm_path.push("temp.s");
        Self::write_asm(asm, &asm_path)?;

        let mut obj_path = CACHE_PATH.clone();
        obj_path.push("temp.o");

        let mut cmd = Command::new("nasm");
        cmd.args(&self.nasm_args)
            .args(["-o", obj_path.to_str().unwrap()])
            .arg("temp.s");
        cmd.spawn()?;

        let mut cmd = Command::new(self.linker);
        cmd.args(&self.linker_args)
            .args(["-o", outfile.to_str().unwrap()])
            .arg(obj_path.to_str().unwrap());
        cmd.spawn()?;

        fs::remove_file(asm_path)?;
        fs::remove_file(obj_path)?;

        Ok(())
    }

    pub fn write_asm(asm: &[String], outfile: &Path) -> Result<(), io::Error> {
        let mut file = fs::File::create(outfile)?;
        file.write_all(asm.join("\n").as_bytes())?;

        Ok(())
    }

    pub fn default() -> &'static Self {
        Self::get_by_string(&DEFAULT_PROFILE).expect("No default profile found")
    }

    pub fn get_by_string(profile: &str) -> Option<&Profile> {
        Self::get_all_profiles()
            .iter()
            .find(|&prof| prof.name == profile)
    }

    pub fn get_all_profiles() -> &'static [Profile] {
        &PROFILES
    }
}
