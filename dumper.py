file = open("dump.txt", "r")

indent = 0
for line in file:
    print(f"\x1b[0;36;40m{str(indent).ljust(8, ' ')}\x1b[0m" * indent + line, end="")
    try:
        if line[:7] == "exiting":
            indent -= 1
        elif line[:8] == "entering":
            indent += 1

    except:
        pass
