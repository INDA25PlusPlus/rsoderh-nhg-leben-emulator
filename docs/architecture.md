# Architecture specification

## Registers

16 16-bit registers:
r0 r1 r2 r3 r4 r5 r6 r7
r8 r9 ra rb rc rd PC SP

r0-rd are general-purpose registers
PC is the program counter register (next instruction pointer)
SP is the stack pointer register

Registers are referred to using their associated emojis. Please consult the
table:

| Register | Emoji |
|----------|-------|
| r0       | 👁    |
| r1       | 🐍    |
| r2       | 🗣    |
| r3       | 🦗    |
| r4       | 🤡    |
| r5       | 🐈    |
| r6       | 👽    |
| r7       | 🦧    |
| r8       | 🍄    |
| r9       | 💔    |
| ra       | 🥀    |
| rb       | 🐊    |
| rc       | 💢    |
| rd       | ✨    |
| PC       | 🔢    |
| SP       | ☝    |

## Assembly instructions

Instructions work by targeting a selection of registers/bits by marking them
with an instruction character. Subsequent instructions can mark another
selection of registers/bits to link them to the previous instruction. For
example, this set of two instructions (at one instruction per line) copy the
values in bits 0-3 of register 🦗 to bits 4-7 of register 🐊:

`🦗 cccc.... ........`
`🐊 ....vvvv ........`

Instructions are one of two types:
- **Multi-register instructions** operate on and modify whole registers
- **Single-register instructions** operate on individual bits in a single
  register

Multi-register instructions are of the form

`🔀 IIIIIIII IIIIIIII`

where the `I`:s are any valid, possibly different, multi-register instruction
characters.

Single-register instructions are of the form

`R iiiiiiii iiiiiiii`

where `R` is any valid register in emoji form (see *Registers*) and the `i`:s
are any valid, possibly different, single-register instruction characters.

The available instructions are analogous for the two types, and are divided
into three classes: **Read**, **Operate** and **Write**. Uppercase letters are
used for multi-register instructions, and lowercase letters are used for
single-register instructions.

### Read instructions

Read instructions are unpaired, meaning they operate only based on the bits in
the instruction where they are specified.

#### Noop (no operation)

. - does nothing.
Example:

`🔀 ....000. 00.00...`
(does not affect registers 0,1,2,3,7,10,13,14,15)

#### Copy

C/c - copies registers/bits to the destination registers/bits specified by the
next instruction. Example:

`🍄 cccc.... ........`
`🍄 ........ v.v.v.v.`
(copies bits 0,1,2,3 to bits 9,11,13,15 respectively)

#### Cut

X/x - cuts the specified registers/bits, and replaces them with zeroes.
Example:

`🔀 ....XXX. XX.XX...`
`🔀 VV.VVV.. ....VV..`
(cuts registers 4,5,6,8,9,11,12, setting them to 0, and pastes their values
into registers 0,1,3,4,5,12,13 respectively)

#### Pointer read

P - for each `P` instruction character, reads the corresponding register's
value, interpret it as an address and copy the value stored at that address.
Only valid for multi-register instructions. Example:

`🔀 PPPP.... ........`
`🔀 ....QQQQ  ........`
(reads the values stored at the addresses specified in registers 0,1,2,3 and
stores the values at the addresses specified in registers 4,5,6,7)

#### Write stdout

W - writes the first 8 bits of the specified registers, and outputs them as
bytes to stdout. Example:

`🔀 WWWW.... ........`
(writes the first 8 bits of registers 0-3, and outputs 4 bytes to stdout)

### Operate instructions

Operate instructions are paired, meaning they must match up with the same
number of the same instruction type in the following instruction.

#### Add

\+ - adds the binary number formed by the specified registers/bits to the
registers/bits specified in the next instruction, truncating any overflowing
bits. If the next instruction is also followed by add instructions, the number 
is instead added to the last consecutive instruction that contains add
instructions. Example:

`🐍 ....+++. +.......`
`👽 ++++.... ........`
`👽 ........ ++++....`
(adds the number formed by bits 4,5,6,8 in 🐍 and the number formed by bits 0-3
in 👽 to the number formed by bits 8-11 in 👽, and stores the result in bits
8-11 in 👽)

Example:

`🔀 ++..++.. ++..++..`
`🔀 ..++..++ ..++..++`
(adds the 128-bit number formed by concatenating the bits in registers
0,1,4,5,8,9,12,13 to the number formed by concatenating the bits in registers
2,3,6,7,10,11,14,15, truncates overflowing bits, and stores the result in
registers 2,3,6,7,10,11,14,15)

Example:

`🐍 ....++++ +.......`
`👽 +++++... ........`
`👽 ........ ++++....`
(number of + instruction characters does not match between instructions; this
is an invalid set of instructions)

#### Subtract

\- - analogously to add, subtracts the number formed by the specified
registers/bits from the number formed by the registers/bits specified in the
next instruction, and stores the result in those registers/bits.

#### Multiply

\* - analogous to add, performs multiplication and truncates overflowing bits

#### Divide

/ - analogous to add, performs integer division

#### And

& - analogous to add, performs bitwise and

#### Or

| - analogous to add, performs bitwise or

#### Xor

^ - analogous to add, performs bitwise xor

### Write instructions

Write instructions set the value of the specified registers/bits without
reading any registers/bits

#### Zero

0 - sets the specified registers/bits to all zeroes

#### One

1 - sets the specified registers/bits to all ones

#### Random

\# - sets the specified registers/bits to all random bits

#### Read std

R - for each `R` instruction character, reads a byte from stdin and stores it
in the first 8 bytes of the specified register. Only valid for multi-register
type instructions. Example:

`🔀 RRRR.... ........`
(reads 4 bytes from stdin and stores them in the first 8 bits of registers 0-3)

#### Allocate

A - for each Read-class instruction character in the previous instruction,
allocate a memory block of the size stored in that register, and store the
address of the returned pointer in the specified register. Only valid for
multi-register instructions. Example:

`🔀 C....... ........`
`🔀 ....A... ........`
(allocates a memory block of size determined by the value in register 0, and
stores the address in register 4)


#### Pointer write

Q - for each Read-class instruction character in the previous instruction,
store its value at the address stored in the specified register. Only valid for
multi-register instructions. Example:

`🔀 CCCC.... ........`
`🔀 ....QQQQ ........`
(copies the value in register 0 to the address stored in register 4, 1 to 5, 2
to 6 and 3 to 7)

