use bitflags_lite::{BitFlags, BitFlagsSet};

#[test]
fn test_permissions() {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
    pub struct PermissionsTag;

    pub type Permissions = BitFlags<u32, PermissionsTag>;

    pub trait PermissionsFlags: Sized {
        const NONE: Self;
        const READ: Self;
        const WRITE: Self;
        const EXEC: Self;
    }

    impl PermissionsFlags for Permissions {
        const NONE: Self = Self::from_bits(0).unwrap();
        const READ: Self = Self::from_bits(1 << 0).unwrap();
        const WRITE: Self = Self::from_bits(1 << 1).unwrap();
        const EXEC: Self = Self::from_bits(1 << 2).unwrap();
    }

    impl BitFlagsSet<u32> for PermissionsTag {
        const EMPTY_BITS: u32 = 0;
        const ALL_BITS: u32 = (1 << 3) - 1;
    }

    let permissions = Permissions::READ | Permissions::WRITE | Permissions::EXEC;
    assert!(permissions.contains(Permissions::READ));
    assert!(permissions.contains(Permissions::WRITE));
    assert!(permissions.contains(Permissions::EXEC));

    let permissions = Permissions::READ;
    assert!(permissions.contains(Permissions::READ));
    assert!(!permissions.contains(Permissions::WRITE));
    assert!(!permissions.contains(Permissions::EXEC));
}
