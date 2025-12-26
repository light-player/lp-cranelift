I want to refactor the way we handle source locations in the glsl compiler.

we have span info from glsl-parser, but we need to handle multiple files, intrinsics, etc.

I want a clean system that can handle all of this.

We can put this in frontend/source_map.rs

It will ultimately replace frontend/src_loc.rs

We need to be able to handle:
- real files vs inline files
- intrinsics

Existing work in: 
- lightplayer/crates/lp-glsl/src/src_loc.rs
- and Span from glsl-parser

We need a better way to handle this through the compiler.

Probably something like this:

GlFileId(u32)

GlSourceMap would hold source information:
- files: Map<GlFileId, GlSourceFile>

struct GlFile
- source: GlFileSource
- contents: String
- lineSpans: Vec<Span>

enum GlFileSource
- File(path: String)
- Intrinsic(name: String)
- Inline(name: String) // maybe give a better name? But used for things like tests, or when there isn't an actual filesystem file

GlSourceLoc:
- fileId: GlFileId
- pos: u32

GlSourceSpan
- fileId: GlFileId
- start: u32
- end: u32


We need functions and such to easily use this from the compiler:
- adding files / source
- converting Span to GlSourceSpan etc

Analyze the current code and propose a design.
This plan won't cover _using_ the design, yet, just building it. We'll use it in a future plan.