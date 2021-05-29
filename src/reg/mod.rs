// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

mod arithm;
mod bitwise;
#[allow(clippy::module_inception)]
mod reg;
mod value;

pub use reg::{Reg16, Reg32, Reg8, RegA, RegAR, RegBlockAR, RegR, Registers};
pub use value::{RegVal, Step, Value};
