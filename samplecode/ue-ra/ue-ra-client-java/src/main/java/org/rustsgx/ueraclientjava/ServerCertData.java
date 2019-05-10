package org.rustsgx.ueraclientjava;

import java.util.List;

public class ServerCertData {
    public List<Byte> payload;
    public byte[] pub_k;

    public ServerCertData(List<Byte> payload, byte[] pub_k) {
        this.payload = payload;
        this.pub_k = pub_k;
    }
}