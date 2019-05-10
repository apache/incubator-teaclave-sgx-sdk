package org.rustsgx.ueraclientjava;

import java.util.Arrays;

public class PlatformInfoBlob {
    private int sgx_epid_group_flags;
    private long sgx_tcb_evaluation_flags;
    private long pse_evaluation_flags;
    private String latest_equivalent_tcb_psvn;
    private String latest_pse_isvsvn;
    private String latest_psda_svn;
    private long xeid;
    private long gid;
    private SGXEC256Signature sgx_ec256_signature_t;

    class SGXEC256Signature {
        private String gx;
        private String gy;

        public String getGx() {
            return gx;
        }

        public void setGx(String gx) {
            this.gx = gx;
        }

        public String getGy() {
            return gy;
        }

        public void setGy(String gy) {
            this.gy = gy;
        }
    }

    public int getSgx_epid_group_flags() {
        return sgx_epid_group_flags;
    }

    public void setSgx_epid_group_flags(int sgx_epid_group_flags) {
        this.sgx_epid_group_flags = sgx_epid_group_flags;
    }


    public void setSgx_tcb_evaluation_flags(int sgx_tcb_evaluation_flags) {
        this.sgx_tcb_evaluation_flags = sgx_tcb_evaluation_flags;
    }


    public void setPse_evaluation_flags(int pse_evaluation_flags) {
        this.pse_evaluation_flags = pse_evaluation_flags;
    }

    public String getLatest_equivalent_tcb_psvn() {
        return latest_equivalent_tcb_psvn;
    }

    public void setLatest_equivalent_tcb_psvn(String latest_equivalent_tcb_psvn) {
        this.latest_equivalent_tcb_psvn = latest_equivalent_tcb_psvn;
    }

    public String getLatest_pse_isvsvn() {
        return latest_pse_isvsvn;
    }

    public void setLatest_pse_isvsvn(String latest_pse_isvsvn) {
        this.latest_pse_isvsvn = latest_pse_isvsvn;
    }

    public String getLatest_psda_svn() {
        return latest_psda_svn;
    }

    public void setLatest_psda_svn(String latest_psda_svn) {
        this.latest_psda_svn = latest_psda_svn;
    }


    public void setXeid(int xeid) {
        this.xeid = xeid;
    }

    public long getSgx_tcb_evaluation_flags() {
        return sgx_tcb_evaluation_flags;
    }

    public void setSgx_tcb_evaluation_flags(long sgx_tcb_evaluation_flags) {
        this.sgx_tcb_evaluation_flags = sgx_tcb_evaluation_flags;
    }

    public long getPse_evaluation_flags() {
        return pse_evaluation_flags;
    }

    public void setPse_evaluation_flags(long pse_evaluation_flags) {
        this.pse_evaluation_flags = pse_evaluation_flags;
    }

    public long getXeid() {
        return xeid;
    }

    public void setXeid(long xeid) {
        this.xeid = xeid;
    }

    public long getGid() {
        return gid;
    }

    public void setGid(long gid) {
        this.gid = gid;
    }

    public void setGid(int gid) {
        this.gid = gid;
    }

    public SGXEC256Signature getSgx_ec256_signature_t() {
        return sgx_ec256_signature_t;
    }

    public void setSgx_ec256_signature_t(SGXEC256Signature sgx_ec256_signature_t) {
        this.sgx_ec256_signature_t = sgx_ec256_signature_t;
    }

    public void parsePlatInfo(byte[] piBlobByte, PlatformInfoBlob pfInfo) {
        pfInfo.sgx_ec256_signature_t = new SGXEC256Signature();
        pfInfo.sgx_epid_group_flags = Byte.toUnsignedInt(piBlobByte[0]);
        pfInfo.sgx_tcb_evaluation_flags = computeDec(Arrays.copyOfRange(piBlobByte, 1, 3));
        pfInfo.pse_evaluation_flags = computeDec(Arrays.copyOfRange(piBlobByte, 3, 5));
        pfInfo.latest_equivalent_tcb_psvn = byte2Str(Arrays.copyOfRange(piBlobByte, 5, 23));
        pfInfo.latest_pse_isvsvn = byte2Str(Arrays.copyOfRange(piBlobByte, 23, 25));
        pfInfo.latest_psda_svn = byte2Str(Arrays.copyOfRange(piBlobByte, 25, 29));
        pfInfo.xeid = computeDec(Arrays.copyOfRange(piBlobByte, 29, 33));
        pfInfo.gid = computeDec(Arrays.copyOfRange(piBlobByte, 33, 37));
        pfInfo.sgx_ec256_signature_t.gx = byte2Str(Arrays.copyOfRange(piBlobByte, 37, 69));
        pfInfo.sgx_ec256_signature_t.gy = byte2Str(Arrays.copyOf(piBlobByte, 69));

    }

    public long computeDec(byte[] piBlobSlice) {
        String hexString = new String();
        for (int i = piBlobSlice.length - 1; i >= 0; i--) {
            hexString += CommonUtils.byteToHex(piBlobSlice[i]);
        }
        return Long.parseLong(hexString, 16);
    }

    public String byte2Str(byte[] piBlobSlice) {
        String piBlobStr = new String();
        for (int i = 0; i < piBlobSlice.length; i++) {
            piBlobStr += Byte.toUnsignedInt(piBlobSlice[i]) + ", ";
        }
        return "[" + piBlobStr.substring(0, piBlobStr.length() - 2) + "]";
    }
}
