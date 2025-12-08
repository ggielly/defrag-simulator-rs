use rand::seq::SliceRandom;
use rand::thread_rng;

// --- MS-DOS 6.22 file categories ---

// Main MS-DOS system files
pub const MSDOS_SYSTEM_FILES: &[&str] = &[
    "IO.SYS",
    "MSDOS.SYS",
    "COMMAND.COM",
    "AUTOEXEC.BAT",
    "CONFIG.SYS",
    "DBLSPACE.BIN",
    "DRVSPACE.BIN",
];

// External MS-DOS commands
pub const MSDOS_COMMANDS: &[&str] = &[
    "APPEND.EXE",
    "ATTRIB.EXE",
    "CHKDSK.EXE",
    "CHOICE.COM",
    "DEBUG.EXE",
    "DEFRAG.EXE",
    "DELTREE.EXE",
    "DISKCOMP.COM",
    "DISKCOPY.COM",
    "DOSKEY.COM",
    "DOSSHELL.EXE",
    "EDIT.COM",
    "EDLIN.EXE",
    "EMM386.EXE",
    "EXPAND.EXE",
    "FASTHELP.EXE",
    "FASTOPEN.EXE",
    "FC.EXE",
    "FDISK.EXE",
    "FIND.EXE",
    "FORMAT.COM",
    "GRAFTABL.COM",
    "GRAPHICS.COM",
    "HELP.COM",
    "INTERLNK.EXE",
    "INTERSVR.EXE",
    "KEYB.COM",
    "LABEL.EXE",
    "LOADFIX.COM",
    "MEM.EXE",
    "MEMMAKER.EXE",
    "MODE.COM",
    "MORE.COM",
    "MOVE.EXE",
    "MSAV.EXE",
    "MSBACKUP.EXE",
    "MSD.EXE",
    "NLSFUNC.EXE",
    "POWER.EXE",
    "PRINT.EXE",
    "QBASIC.EXE",
    "REPLACE.EXE",
    "RESTORE.EXE",
    "SCANDISK.EXE",
    "SETVER.EXE",
    "SHARE.EXE",
    "SIZER.EXE",
    "SMARTDRV.EXE",
    "SORT.EXE",
    "SUBST.EXE",
    "SYS.COM",
    "TREE.COM",
    "UNDELETE.EXE",
    "UNFORMAT.COM",
    "VSAFE.COM",
    "XCOPY.EXE",
];

// Common drivers
pub const DRIVERS_FILES: &[&str] = &[
    "HIMEM.SYS",
    "EMM386.EXE",
    "RAMDRIVE.SYS",
    "ANSI.SYS",
    "COUNTRY.SYS",
    "DISPLAY.SYS",
    "DRIVER.SYS",
    "KEYBOARD.SYS",
    "PRINTER.SYS",
    "MOUSE.COM",
    "MOUSE.SYS",
    "SETVER.EXE",
    "SMARTDRV.EXE",
];

// Configuration files and examples
pub const CONFIG_FILES: &[&str] = &[
    "AUTOEXEC.UMB",
    "AUTOEXEC.DOS",
    "CONFIG.UMB",
    "CONFIG.DOS",
    "NETWORKS.TXT",
    "README.TXT",
    "APPS.HLP",
    "MSBACKUP.INI",
    "MSBACKUP.CFG",
    "MSAV.INI",
    "DEFRAG.INI",
    "QBASIC.INI",
];

// --- Popular application file categories ---

// WordPerfect 5.1
pub const WP51_FILES: &[&str] = &[
    "WP\\WP.EXE",
    "WP\\WP.FIL",
    "WP\\STANDARD.PRS",
    "WP\\MACROS.WPM",
    "WP\\STYLES.STY",
    "WP\\GRAPHCNV.EXE",
    "WP\\WPINFO.TXT",
];

// Lotus 1-2-3
pub const LOTUS123_FILES: &[&str] = &[
    "123\\123.EXE",
    "123\\123.CNF",
    "123\\WYSIWYG.CNF",
    "123\\DEFAULT.FNT",
    "123\\SAMPLES\\SALES.WK1",
    "123\\SAMPLES\\FINANCE.WK1",
];

// dBase IV
pub const DBASE_IV_FILES: &[&str] = &[
    "DBASE\\DBASE.EXE",
    "DBASE\\DBASE.RES",
    "DBASE\\SQLHOME\\SQL.EXE",
    "DBASE\\SAMPLES\\CLIENTS.DBF",
    "DBASE\\SAMPLES\\ORDERS.DBF",
    "DBASE\\SAMPLES\\ITEMS.NDX",
    "DBASE\\TUTORIAL\\TUTOR.DBF",
];

// Norton Utilities
pub const NORTON_UTILS_FILES: &[&str] = &[
    "NORTON\\NU.EXE",
    "NORTON\\SPEEDISK.EXE",
    "NORTON\\DISKDOC.EXE",
    "NORTON\\UNERASE.EXE",
    "NORTON\\SYSINFO.EXE",
];

// --- Popular DOS Game Categories ---

// Doom
pub const DOOM_FILES: &[&str] = &[
    "DOOM\\DOOM.EXE",
    "DOOM\\DOOM.WAD",
    "DOOM\\SETUP.EXE",
    "DOOM\\DEHACKED.EXE",
    "DOOM\\DM.WAD",
    "DOOM\\DOOM2.WAD",
    "DOOM\\TNT.WAD",
    "DOOM\\PLUTONIA.WAD",
];

// Duke Nukem 3D
pub const DUKE3D_FILES: &[&str] = &[
    "DUKE3D\\DUKE3D.EXE",
    "DUKE3D\\DUKE.RTS",
    "DUKE3D\\SETUP.EXE",
    "DUKE3D\\USER.CON",
    "DUKE3D\\GAME.CON",
    "DUKE3D\\DUKEMATCH.CON",
];

// Wolfenstein 3D
pub const WOLF3D_FILES: &[&str] = &[
    "WOLF3D\\WOLF3D.EXE",
    "WOLF3D\\VGAHEAD.V3D",
    "WOLF3D\\VGAGRAPH.V3D",
    "WOLF3D\\VSWAP.V3D",
    "WOLF3D\\AUDIO.W3D",
    "WOLF3D\\MAPHEAD.W3D",
];

// Civilization
pub const CIV_FILES: &[&str] = &[
    "CIV\\CIV.EXE",
    "CIV\\MAP.GIF",
    "CIV\\CIV.SVE",
    "CIV\\DIPLOMAT.TXT",
    "CIV\\INTRO.EXE",
];

// The Secret of Monkey Island
pub const MONKEY_ISLAND_FILES: &[&str] = &[
    "MONKEY\\MONKEY.EXE",
    "MONKEY\\000.LFL",
    "MONKEY\\901.LFL",
    "MONKEY\\DISK01.LEC",
];

// --- Windows 3.1 / 3.11 
pub const WINDOWS_31_FILES: &[&str] = &[
    "WINDOWS\\WIN.COM",
    "WINDOWS\\SYSTEM.INI",
    "WINDOWS\\WIN.INI",
    "WINDOWS\\PROGMAN.EXE",
    "WINDOWS\\PROGMAN.INI",
    "WINDOWS\\SYSTEM\\GDI.EXE",
    "WINDOWS\\SYSTEM\\USER.EXE",
    "WINDOWS\\SYSTEM\\KRNL386.EXE",
    "WINDOWS\\SYSTEM\\VGA.DRV",
    "WINDOWS\\SYSTEM\\COMM.DRV",
    "WINDOWS\\SYSTEM\\MMSOUND.DRV",
    "WINDOWS\\SYSTEM\\SHELL.DLL",
    "WINDOWS\\SYSTEM\\WINFILE.EXE",
    "WINDOWS\\SOL.EXE",
    "WINDOWS\\WINMINE.EXE",
    "WINDOWS\\CLOCK.EXE",
    "WINDOWS\\WRITE.EXE",
    "WINDOWS\\NOTEPAD.EXE",
    "WINDOWS\\REGEDIT.EXE",
    "WINDOWS\\CONTROL.EXE",
    "WINDOWS\\CALC.EXE",
    "WINDOWS\\CARDFILE.EXE",
];

/// Provides unique, random filenames from the DOS era.
pub struct DosFileProvider {
    remaining_files: Vec<&'static str>,
}

impl DosFileProvider {
    /// Creates a new provider, populates it with all file lists,
    /// and shuffles the list for unique random distribution.
    pub fn new() -> Self {
        let mut remaining_files = Vec::new();

        // Add all file categories
        remaining_files.extend_from_slice(MSDOS_SYSTEM_FILES);
        remaining_files.extend_from_slice(MSDOS_COMMANDS);
        remaining_files.extend_from_slice(DRIVERS_FILES);
        remaining_files.extend_from_slice(CONFIG_FILES);
        remaining_files.extend_from_slice(WP51_FILES);
        remaining_files.extend_from_slice(LOTUS123_FILES);
        remaining_files.extend_from_slice(DBASE_IV_FILES);
        remaining_files.extend_from_slice(NORTON_UTILS_FILES);
        remaining_files.extend_from_slice(DOOM_FILES);
        remaining_files.extend_from_slice(DUKE3D_FILES);
        remaining_files.extend_from_slice(WOLF3D_FILES);
        remaining_files.extend_from_slice(CIV_FILES);
        remaining_files.extend_from_slice(MONKEY_ISLAND_FILES);
        remaining_files.extend_from_slice(WINDOWS_31_FILES);

        // Shuffle the list to get a random order
        remaining_files.shuffle(&mut thread_rng());

        Self { remaining_files }
    }

    /// Returns a unique filename from the list.
    /// Returns `None` if all files have been used.
    pub fn get_random_filename(&mut self) -> Option<String> {
        self.remaining_files.pop().map(|s| s.to_string())
    }
}
