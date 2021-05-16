// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify::num::u5;

use crate::registers::{Reg, Reg32, Reg8, RegA, RegR};

#[non_exhaustive]
pub enum Instruction {
    // #[value = 0b00_000_000]
    ControlFlow(ControlFlowOp),

    // #[value = 0b00_001_000]
    Register(RegisterOp),

    // #[value = 0b00_010_000]
    Cmp(CmpOp),

    // #[value = 0b00_100_000]
    Arithmetic(ArithmeticOp),

    // #[value = 0b00_101_000]
    Bitwise(BitwiseOp),

    // #[value = 0b00_110_000]
    Bytes(BytesOp),

    // #[value = 0b01_000_000]
    Digest(DigestOp),

    // #[value = 0b01_001_000]
    Secp256k1(SecpOp),

    // #[value = 0b01_001_100]
    Ed25519(Ed25519Op),

    // #[value = 0b10_000_000]
    ExtensionCodes,

    // #[value = 0b11_111_111]
    Nop,
}

pub enum ControlFlowOp {
    /// Completes program execution writing `false` to `st0` (indicating
    /// program failure)
    // #[value = 0b000]
    Fail,

    /// Completes program execution writing `true` to `st0` (indicating program
    /// success)
    // #[value = 0b001]
    Succ,

    /// Unconditionally jumps to an offset. Increments `cy0`.
    // #[value = 0b010]
    Jmp(u16),

    /// Jumps to an offset if `st0` == true, otherwise does nothing. Increments
    /// `cy0`.
    // #[value = 0b011]
    Jif(u16),

    /// Jumps to other location in the current code with ability to return
    /// back (calls a subroutine). Increments `cy0` and pushes offset of the
    /// instruction which follows current one to `cs0`.
    Routine(u16),

    /// Calls code from an external library identified by the hash of its code.
    /// Increments `cy0` and `cp0` and pushes offset of the instruction which
    /// follows current one to `cs0`.
    Call([u8; 32], u16),

    /// Passes execution to other library without an option to return.
    /// Does not increments `cy0` and `cp0` counters and does not add anything
    /// to the call stack `cs0`.
    Exec([u8; 32], u16),

    /// Returns execution flow to the previous location from the top of `cs0`.
    /// Does not change value in `cy0`. Decrements `cp0`.
    Ret,
}

pub enum RegisterOp {
    /// Swap operation. If the value does not fit destination bit dimensions
    /// truncates the most significant bits until they fit.
    Swp(
        Reg,
        Reg32,
        Reg,
        Reg32,
        /** Fill extra bits with highest bit for first value */ bool,
        /** Fill extra bits with highest bit for second value */ bool,
    ),
    /// Duplicates value from low 16 registers to high 16 registers
    Mov(
        Reg,
        Reg32,
        Reg,
        Reg32,
        /// Flag indicating whether the register value should be duplicated
        /// (`true`) or just moved (`false`)
        bool,
        /** Fill extra bits with highest bit */ bool,
    ),

    /// Sets register value to zero
    Zeroa(RegA, Reg32),
    Zeror(RegR, Reg32),

    /// Cleans a value of a register (sets it to undefined state)
    Cleana(RegA, Reg32),
    Cleanr(RegR, Reg32),

    Puta(RegA, Reg32, u16, [u8; 1024]),
    Putr(RegR, Reg32, u16, [u8; 1024]),
}

pub enum CmpOp {
    /// Compares value of two arithmetic (`A`) registers putting result into
    /// `cm0`
    // #[value = 0b110] // 3 + 5 + 3 + 5 => 16 bits
    Cmpa(RegA, Reg32, RegA, Reg32),

    /// Compares value of two non-arithmetic (`R`) registers putting result
    /// into `cm0`
    // #[value = 0b111]
    Cmpr(RegR, Reg32, RegR, Reg32),

    /// Checks equality of value in two arithmetic (`A`) registers putting
    /// result into `st0`
    // #[value = 0b100]
    Eqa(RegA, Reg32, RegA, Reg32),

    /// Checks equality of value in two non-arithmetic (`R`) registers putting
    /// result into `st0`
    // #[value = 0b101]
    Eqr(RegR, Reg32, RegR, Reg32),

    /// Measures bit length of a value in one fo the registers putting result
    /// to `a16[0]`
    Lena(RegA, Reg32, Reg32),
    Lenr(RegA, Reg32, Reg32),

    /// Counts number of `1` bits in register putting result to `a16[0]`
    /// register
    Cnta(RegA, Reg32, Reg32),
    Cntr(RegR, Reg32, Reg32),
}

pub enum Arithmetics {
    IntChecked(bool),
    IntUnchecked(bool),
    IntArbitraryPrecision(bool),
    Float,
    FloatArbitraryPrecision,
}

pub enum ArithmeticOp {
    Neg(RegA, Reg32),                     // 3 + 5 = 8 bits
    Inc(Arithmetics, RegA, Reg32, u5),    // Increases value on a given step
    Add(Arithmetics, RegA, Reg32, Reg32), // 3 + 3 + 5 + 5  => 16 bits
    Sub(Arithmetics, RegA, Reg32, Reg32),
    Mul(Arithmetics, RegA, Reg32, Reg32),
    Div(Arithmetics, RegA, Reg32, Reg32),
    Mod(RegA, Reg32),              // 3 + 5 = 8 bits
    Abs(RegA, Reg32, RegA, Reg32), // 3 + 5 + 3 + 5 => 16 bits
}

pub enum BitwiseOp {
    And(
        RegA,
        Reg32,
        Reg32,
        /// Operation destination, only first 8 registers
        Reg8,
    ),
    Or(RegA, Reg32, Reg32, Reg8),
    Xor(RegA, Reg32, Reg32, Reg8),

    Not(RegA, Reg32),

    Shl(RegA, Reg32, Reg32 /* Always `a8` */, Reg8),
    Shr(RegA, Reg32, Reg32, Reg8),
    /// Shift-cycle left
    Scl(RegA, Reg32, Reg32, Reg8),
    /// Shift-cycle right
    Scr(RegA, Reg32, Reg32, Reg8),
}

pub enum BytesOp {
    Puts(/** `s` register index */ u8, u16, [u8; u16::MAX as usize]),

    Movs(/** `s` register index */ u8, /** `s` register index */ u8),

    Swps(/** `s` register index */ u8, /** `s` register index */ u8),

    Fill(
        /** `s` register index */ u8,
        /** from */ u16,
        /** to */ u16,
        /** value */ u8,
    ),

    /// Returns length of the string
    Lens(/** `s` register index */ u8),

    /// Counts number of byte occurrences within the string
    Counts(/** `s` register index */ u8, /** byte to count */ u8),

    /// Compares two strings from two registers, putting result into `cm0`
    Cmps(u8, u8),

    /// Computes length of the fragment shared between two strings
    Common(u8, u8),

    /// Counts number of occurrences of one string within another putting
    /// result to `a16[0]`
    Find(
        /** `s` register with string */ u8,
        /** `s` register with matching fragment */ u8,
    ),

    /// Extracts value into a register
    Exta(RegA, Reg32, /** `s` register index */ u8, /** offset */ u16),
    Extr(RegR, Reg32, /** `s` register index */ u8, /** offset */ u16),

    Join(
        /** Source 1 */ u8,
        /** Source 2 */ u8,
        /** Destination */ u8,
    ),
    Split(
        /** Source */ u8,
        /** Offset */ u16,
        /** Destination 1 */ u8,
        /** Destination 2 */ u8,
    ),
    Ins(
        /** Insert from register */ u8,
        /** Insert to register */ u8,
        /** Offset for insert place */ u16,
    ),
    Del(
        /** Register index */ u8,
        /** Delete from */ u16,
        /** Delete to */ u16,
    ),
    /// Translocates fragment of bytestring into a register
    Transl(
        /** Source */ u8,
        /** Start from */ u16,
        /** End at */ u16,
        /** Index to put translocated portion */ u8,
    ),
}

#[non_exhaustive]
pub enum DigestOp {
    Ripemd(
        /** Which of `a16` registers contain start offset */ Reg32,
        /** Index of string register */ Reg32,
        /** Index of `r160` register to save result to */ Reg32,
        /** Clear string register after operation */ bool,
    ),
    Sha2(
        /** Which of `a16` registers contain start offset */ Reg32,
        /** Index of string register */ Reg32,
        /** Index of `r160` register to save result to */ Reg32,
        /** Clear string register after operation */ bool,
    ),
}

pub enum SecpOp {
    Gen(
        /** Register containing scalar */ Reg32,
        /** Destination register to put G * scalar */ Reg8,
    ),
    Mul(
        /** Use `a` or `r` register as scalar source */ bool,
        /** Scalar register index */ Reg32,
        /** Source `r` register index containing EC point */ Reg32,
        /** Destination `r` register index */ Reg32,
    ),
    Add(
        /** Allow overflows */ bool,
        /** Source 1 */ Reg32,
        /** Source 2 */ Reg32,
        /** Source 3 */ Reg32,
    ),
    Neg(
        /** Register hilding EC point to negate */ Reg32,
        /** Destination register */ Reg8,
    ),
}

pub enum Ed25519Op {
    Gen(
        /** Register containing scalar */ Reg32,
        /** Destination register to put G * scalar */ Reg8,
    ),
    Mul(
        /** Use `a` or `r` register as scalar source */ bool,
        /** Scalar register index */ Reg32,
        /** Source `r` register index containing EC point */ Reg32,
        /** Destination `r` register index */ Reg32,
    ),
    Add(
        /** Allow overflows */ bool,
        /** Source 1 */ Reg32,
        /** Source 2 */ Reg32,
        /** Source 3 */ Reg32,
    ),
    Neg(
        /** Register hilding EC point to negate */ Reg32,
        /** Destination register */ Reg8,
    ),
}