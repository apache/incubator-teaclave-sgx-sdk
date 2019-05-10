package org.rustsgx.ueraclientjava;

import java.io.File;
import java.util.ArrayList;
import java.util.List;

public class CommonUtils {
    public static List<Byte> string2BytesList(String[] strings) {
        ArrayList<Byte> arrayList = new ArrayList<Byte>();
        for (int i = 0; i < strings.length; i++) {
            int intVal = Integer.decode(strings[i]);
            arrayList.add(Byte.valueOf((byte) intVal));
        }
        return arrayList;
    }

    public static int getIndexOf(List<Byte> b, List<Byte> bb) {
        if (b == null || bb == null || b.size() == 0 || bb.size() == 0 || b.size() < bb.size())
            return -1;

        int i, j;
        for (i = 0; i < b.size() - bb.size() + 1; i++) {
            if (b.get(i) == bb.get(0)) {
                for (j = 1; j < bb.size(); j++) {
                    if (b.get(i + j) != bb.get(j))
                        break;
                }
                if (j == bb.size())
                    return i;
            }
        }
        return -1;
    }

    public static byte hexToByte(String inHex) {
        return (byte) Integer.parseInt(inHex, 16);
    }

    public static String byteToHex(byte b) {
        String hex = Integer.toHexString(b & 0xFF);
        if (hex.length() == 1) {
            hex = "0" + hex;
        }
        return hex;
    }

    public static byte[] list2array(List<Byte> list) {
        byte[] bytes = new byte[list.size()];
        for (int i = 0; i < list.size(); i++) {
            bytes[i] = list.get(i);
        }
        return bytes;
    }

    public static void printCert(byte[] rawByte) {
        System.out.print("---received-server cert: [Certificate(b\"");
        for (int i = 0; i < rawByte.length; i++) {
            char c = (char) (Byte.toUnsignedInt(rawByte[i]));
            if (c == '\n') {
                System.out.print("\\n");
            } else if (c == '\r') {
                System.out.print("\\r");
            } else if (c == '\t') {
                System.out.print("\\t");
            } else if (c == '\\' || c == '"') {
                System.out.printf("\\%c", c);
            } else if (Byte.toUnsignedInt(rawByte[i]) >= 32 && Byte.toUnsignedInt(rawByte[i]) < 127) {
                System.out.printf("%c", c);
            } else {
                System.out.printf("\\x%02x", rawByte[i]);
            }
        }
        System.out.println("\")]");
    }
}