# Debug Parse Issue

Input: "Point = { x: i32, y: i32 }"

Error: "Expected '(' for function parameters" at column 11 (which is where "i32" starts)

The error is coming from parse_function but we should be calling parse_struct.

Let me trace:
1. Parser sees "Point" (identifier)
2. Peek token is "="
3. Goes into the block at line 15
4. Since peek_token is not "<", goes to line 35
5. Advances to skip "=" (line 36)
6. Current token is now "{"
7. is_struct = true (line 39)
8. Restores state (lines 43-48)
9. Current token is back to "Point"
10. Calls parse_struct (line 51)
11. parse_struct expects "Point" as current token - OK
12. parse_struct advances past "Point"
13. Current token is now "="
14. parse_struct checks for "<" for generics - no
15. parse_struct expects "=" - YES
16. parse_struct advances past "="
17. Current token is now "{"
18. parse_struct expects "{" - YES
19. parse_struct advances past "{"
20. Current token is now "x"
21. parse_struct parses field name "x" - OK
22. parse_struct advances past "x"
23. Current token is now ":"
24. parse_struct expects ":" - YES
25. parse_struct advances past ":"
26. Current token is now "i32"
27. parse_struct calls parse_type()
28. parse_type probably fails?

The error message says "Expected '(' for function parameters" which is from parse_function.
parse_function is at line 29 expecting '('

Ah! The issue might be in parse_type. If parse_type fails or doesn't recognize "i32", it might somehow trigger parse_function.