# rsoderh-jonsh-leben-emulator

Semi-functioning Intel 8080 assembler and emulator.

## How to use

`<EXE> [<file-path>]` - Assemble and run the file at `<file-path>`. If no file path is specified, run an empty emulator instance.

## Examples

Example programs are provided under `./examples`.

## Intel 8080 implementation

Currently supports all standard instructions and data statements, as well as some pseudo-instructions (see below). Input/output instructions use stdin/stdout (see below). Hardware interrupts are not supported.

### Stack

The stack pointer defaults to the value `0`, which means that any `PUSH` instruction will cause a stack overflow error. Thus, it is recommended to set the stack pointer register at the start of the program, for example by using the `LXI` instruction (`LXI SP, 0FFFFH`).

### Labels

Label names may be 1-5 characters long, and can contain any capital alphabetical or numerical characters, except for the first character, which may be a capital alphabetical character or any of the characters `@` and `?`. Examples:

Valid label names: `@MAIN`, `?JUMP`, `ABCDE`, `S1234`, `@`, `ABC`

Invalid label names: `12345`, `ABCDEFGH`, `A@`, `?JMP?`, `main`, `?jump`, `T_AB`, ``

### Numerical values

Numerical values can be specified in decimal, octal or hexadecimal format by appending a base-dependent character to the digit string:
- Decimal: no character (`12345`)
- Hexadecimal: `H` (`0FFFFH`)
- Octal: `O` or `Q` (`17Q`)

It is recommended to prefix all hexadecimal numerical values with `0` to ensure that they are not parsed as labels.

### Standard instruction set

For a complete list of all available instructions, see the Intel 8080 documentation / programmers's guide. Instruction arguments may only be provided in the form of register names, constant values (in decimal/octal/hexadecimal) or, where applicable, labels. The instruction format is otherwise as specified in the Intel 8080 documentation.

### I/O (`IN`, `OUT`)

The input/output device number specified in the instruction is mapped as follows:

`IN 0`: Reads one byte from stdin, and stores it in the accumulator register.

`IN x` for all other `x`: Sets the accumulator register to `0`.

`OUT 0`: Writes the byte stored in the accumulator register to stdout.

`OUT 1`: Writes the byte stored in the accumulator register to stdout formatted as a decimal number.

`OUT 2`: Writes the word stored in the HL register pair to stdout formatted as a decimal number.

`OUT x` for all other `x`: No-op.

### Data statements (`DB`, `DW`, `DS`)

Data statements define data to be stored at a specified memory location.

#### Define byte(s) (`DB`)

The provided argument may be either a numerical constant or a single quote-enclosed string constant. Examples:

`DB 012H`: Stores the value `0x12` at the address of the statement.

`DB 'Hello, world!'`: Stores the given string in a section of bytes starting at the address of the statement, encoded in ASCII.

#### Define word (`DW`)

The provided argument may be either a numerical constant or the name of a label. Examples:

`DW 12345`: Stores the value `12345` in the two-byte sequence starting at the address of the statement.

`DW LABEL`: Stores the address of the label `LABEL` in the two-byte sequence starting at the address of the statement.

#### Define storage (`DS`)

The provided argument must be a numerical constant, specifying the size, in bytes, of the storage section. Example:

`DS 100Q`: Allocates a section of size `0o100` as data storage, starting at the address of the statement.

### Origin (`ORG`) pseudo-instruction

Determines the program's starting address. May only be put at the start of the program. Example:

`ORG 0100H`: Shifts all instructions' and data statements' addresses by the number `0x100`.

### End of assembly (`END`) pseudo-instrution

Must appear at the very end of the program, and may not appear more than once. Signifies the end of the program.

### Unsupported pseudo-instructions

The following pseudo-instructions documented in the Intel 8080 specification are not currently supported:

`EQU`, `SET`, `IF`, `ENDIF`, `MACRO`, `ENDM`
