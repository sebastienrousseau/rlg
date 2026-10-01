# The README demo, rendered by `make demo` (VHS: charmbracelet/vhs).
# It runs the real `rlg` and `rlg-report` binaries, built by the make
# target, against the fixture in .github/demo/app.log.

Output .github/demo.gif

Set Shell "bash"
Set FontSize 15
Set Width 1280
Set Height 680
Set Padding 20
Set TypingSpeed 35ms
Set Theme "Catppuccin Mocha"

Hide
Type "export PS1='$ ' && clear"
Enter
Show

Type "# Twelve rlg records from a service"
Enter
Type "head -n 3 .github/demo/app.log"
Enter
Sleep 2s

Type "# Errors and worse, converted to Elastic Common Schema"
Enter
Type "rlg --min-level error --format ecs .github/demo/app.log"
Enter
Sleep 3s

Type "clear"
Enter
Type "# The same file, summarised"
Enter
Type "rlg-report .github/demo/app.log | head -n 30"
Enter
Sleep 4s
