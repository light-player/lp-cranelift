We need a good way of tracking source information.

Existing work in: 
- lightplayer/crates/lp-glsl/src/src_loc.rs
- and Span from glsl-parser


We need a better way to handle this through the compiler.

Probably something like this:

GlFileId(u32)

GlSourceMap would hold source information:
- files: Map<GlFileId, GlSourceFile>

struct GlSourceFile
- source: String
- lineSpans: Vec<Span>

GlSourceLoc:
- fileId: GlFileId
- pos: u32

GlSourceSpan
- fileId: GlFileId
- start: u32
- end: u32