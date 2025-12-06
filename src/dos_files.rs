use rand::seq::SliceRandom;

// Liste de fichiers système et commandes typiques de MS-DOS 6.22
pub const MS_DOS_FILES: &[&str] = &[
    "IO.SYS", "MSDOS.SYS", "COMMAND.COM", "AUTOEXEC.BAT", "CONFIG.SYS",
    "EDIT.COM", "QBASIC.EXE", "FDISK.EXE", "FORMAT.COM", "CHKDSK.EXE",
    "MEM.EXE", "ATTRIB.EXE", "DEFRAG.EXE", "SCANDISK.EXE", "HIMEM.SYS",
    "EMM386.EXE", "SMARTDRV.EXE", "MOUSE.COM", "DOSSHELL.EXE", "XCOPY.EXE",
];

// Liste de fichiers typiques d'une installation de Windows 3.11
pub const WINDOWS_311_FILES: &[&str] = &[
    "WINDOWS\\WIN.COM", "WINDOWS\\SYSTEM.INI", "WINDOWS\\WIN.INI",
    "WINDOWS\\SYSTEM\\GDI.EXE", "WINDOWS\\SYSTEM\\USER.EXE", "WINDOWS\\SYSTEM\\KRNL386.EXE",
    "WINDOWS\\PROGMAN.EXE", "WINDOWS\\SOL.EXE", "WINDOWS\\WINMINE.EXE", "WINDOWS\\CLOCK.EXE",
    "WINDOWS\\SYSTEM\\VGA.DRV", "WINDOWS\\SYSTEM\\COMM.DRV", "WINDOWS\\SYSTEM\\MMSOUND.DRV",
    "WINDOWS\\WRITE.EXE", "WINDOWS\\NOTEPAD.EXE", "WINDOWS\\REGEDIT.EXE",
];

// Liste de fichiers typiques d'une installation de dBase IV
pub const DBASE_IV_FILES: &[&str] = &[
    "DBASE\\DBASE.EXE", "DBASE\\DBASE.RES", "DBASE\\SQLHOME\\SQL.EXE",
    "DBASE\\SAMPLES\\CLIENTS.DBF", "DBASE\\SAMPLES\\ORDERS.DBF", "DBASE\\SAMPLES\\ITEMS.NDX",
    "DBASE\\TUTORIAL\\TUTOR.DBF",
];

// Liste de fichiers de jeux DOS populaires
pub const DOS_GAMES_FILES: &[&str] = &[
    "DOOM\\DOOM.EXE", "DOOM\\DOOM.WAD", "DOOM\\SETUP.EXE",
    "DUKE3D\\DUKE3D.EXE", "DUKE3D\\DUKE.RTS",
    "CIV\\CIV.EXE", "CIV\\MAP.GIF",
];

/// Fournit des noms de fichiers de l'ère DOS de manière aléatoire.
pub struct DosFileProvider {
    all_files: Vec<&'static str>,
}

impl DosFileProvider {
    /// Crée un nouveau fournisseur et le remplit avec toutes les listes de fichiers.
    pub fn new() -> Self {
        let mut all_files = Vec::new();
        all_files.extend_from_slice(MS_DOS_FILES);
        all_files.extend_from_slice(WINDOWS_311_FILES);
        all_files.extend_from_slice(DBASE_IV_FILES);
        all_files.extend_from_slice(DOS_GAMES_FILES);
        Self { all_files }
    }

    /// Retourne un nom de fichier aléatoire à partir de la collection complète.
    pub fn get_random_filename(&self) -> Option<String> {
        let mut rng = rand::thread_rng();
        self.all_files.choose(&mut rng).map(|s| s.to_string())
    }
}
