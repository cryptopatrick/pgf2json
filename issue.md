Based on the debug output, it's having trouble parsing
   the second concrete grammar in the PGF file. From the
   logs:

  PARSER: Processing concrete 2 of 2
  PARSER: Read suspicious language name 's' for concrete
   2, attempting ZeroSwe recovery
  PARSER: Reading concrete for language: CId("s")
  PARSER: Failed to parse concrete for CId("s"):
  DeserializeError { offset: 580, message: "List length
  268435455 too large at pos 580 - likely parsing error"
   }

  The parser is:
  1. Successfully parsing the first concrete grammar
  (HelloEng)
  2. Encountering issues with the second concrete
  grammar, where it reads a suspicious language name 's'
   instead of the expected HelloIta
  3. Attempting some kind of "ZeroSwe recovery"
  4. Failing with a deserialization error about an
  impossibly large list length (268435455), indicating
  data corruption or parsing offset issues

  This suggests the PGF file itself may be corrupted or
  there's a bug in the pgf2json 0.2.0 parser when
  handling the second concrete grammar. The parser stops
   after the error but returns what it successfully
  parsed (just HelloEng).

  The HelloIta concrete grammar is not being parsed
  correctly from the PGF file, which is why the
  resulting JSON only contains HelloEng in the concretes
   section.
