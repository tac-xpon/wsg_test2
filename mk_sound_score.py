import binascii
import sys

SIZE = 8192
CRC = 0xae9d06d9
OFFSET = 0xe000
TBL = 0xe5e5

f = open('td1_4.1k', 'rb')
rom = f.read()
print("file size=%d crc=%#x" % (len(rom), binascii.crc32(rom)))
if len(rom) == SIZE and binascii.crc32(rom) == CRC:
    print("valid")
else:
    exit()

for i in range(0, 0x20):
    adr = rom[TBL + i * 2 - OFFSET] * 256 + rom[TBL + i * 2 + 1 - OFFSET]
    # print("%2d:%#x" % (i, adr))
    sound_name = "SOUND_" + format(i, "X").zfill(2)
    j = 0
    tbl_list = {}
    sound_def_lines = []
    while rom[adr + j - OFFSET] != 0xe0:
        part_start_adr = rom[adr + j + 0 - OFFSET] * 256 + rom[adr + j + 1 - OFFSET]
        tbl_list.setdefault(part_start_adr);
        scale = rom[adr + j + 2 - OFFSET]
        j += 3
        line = format("(%s_%X, &SCALE_%d)," % (sound_name, part_start_adr, scale))
        sound_def_lines.append(line)
    for part_start_adr in tbl_list.keys():
        current_adr = part_start_adr
        part_bottom_adr = current_adr
        f4_loop_dic = {}
        f5_loop_dic = {}
        continuation = True
        part_score = bytearray()
        while continuation:
            part_bottom_adr = max(part_bottom_adr, current_adr)
            r0 = rom[current_adr + 0 - OFFSET]
            # input("%#x:%s" % (current_adr, format(r0, "x").zfill(2)))
            if r0 >= 0x00 and r0 <= 0xcf:
                part_score.append(r0)
                part_score.append(rom[current_adr + 1 - OFFSET])
                current_adr += 2
                continue
            if r0 >= 0xf0 and r0 <= 0xf2:
                part_score.append(r0)
                part_score.append(rom[current_adr + 1 - OFFSET])
                current_adr += 2
                continue
            if r0 == 0xf3:
                part_score.append(r0)
                current_adr += 1
                continuation = False
                continue
            if r0 == 0xf4:
                r1 = rom[current_adr + 1 - OFFSET]
                j0 = rom[current_adr + 2 - OFFSET]
                j1 = rom[current_adr + 3 - OFFSET]
                jump_adr = j0 * 256 + j1
                work_f4 = f4_loop_dic.get(current_adr, 0)
                work_f4 += 1
                if work_f4 < r1:
                    f4_loop_dic[current_adr] = work_f4
                    # if jump_adr < part_start_adr:
                        # print("// jump under range!! %#x to %#x(start is %#x)" % (current_adr, jump_adr, part_start_adr))
                    current_adr = jump_adr
                    continue
                f4_loop_dic[current_adr] = 0
                current_adr += 4
                continue
            if r0 == 0xf5:
                r1 = rom[current_adr + 1 - OFFSET]
                j0 = rom[current_adr + 2 - OFFSET]
                j1 = rom[current_adr + 3 - OFFSET]
                jump_adr = j0 * 256 + j1
                work_f5 = f5_loop_dic.get(current_adr, 0)
                work_f5 += 1
                if work_f5 == r1:
                    f5_loop_dic[current_adr] = 0
                    # if jump_adr < part_start_adr:
                        # print("// jump under range!! %#x to %#x(start is %#x)" % (current_adr, jump_adr, part_start_adr))
                    current_adr = jump_adr
                    continue
                f5_loop_dic[current_adr] = work_f5
                current_adr += 4
                continue
            if r0 == 0xf6:
                j0 = rom[current_adr + 1 - OFFSET]
                j1 = rom[current_adr + 2 - OFFSET]
                jump_adr = j0 * 256 + j1
                # if jump_adr < part_start_adr:
                    # print("// jump under range!! %#x to %#x(start is %#x)" % (current_adr, jump_adr, part_start_adr))
                current_adr = jump_adr
                continue
            print("// invalid op. %#x in %#x" % (r0, current_adr))
            current_adr += 2
        part_bottom_adr = max(part_bottom_adr, current_adr)
        n = 0
        print("const %s_%X: &[u8] = &[" % (sound_name, part_start_adr))
        for b in part_score:
            if n % 16 == 0:
                print("\t", end = "")
            else:
                print(" ", end = "")
            print("0x%s," % format(b, "x").zfill(2), end = "")
            n += 1
            if n % 16 == 0:
                print()
        if n % 16 != 0:
            print()
        print("];")
    print("const %s: &[(&[u8], &ScaleSet)] = &[" % sound_name)
    for line in sound_def_lines:
        print("\t%s" % line)
    print("];")
    #input("%s was finished" % sound_name)
    print()






