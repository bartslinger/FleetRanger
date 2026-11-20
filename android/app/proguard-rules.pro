# Error reports show real class and method names and line numbers: R8 still
# shrinks, but doesn't rename (the tracker can't map names back).
-dontobfuscate
-keepattributes SourceFile,LineNumberTable
