import { NetworkName } from "../../common/config/index";

export type NetworkItem = {
  [K in NetworkName]?: string;
};

export type CollectionItem = {
  [K in CollectionName]?: NetworkItem;
};

export interface CollectionConfig {
  MAINNET?: CollectionItem;
  TESTNET?: CollectionItem;
}

export const LEGACY_CHAIN_ID_LIST = ["stargaze-0", "stargaze-2"];

export type CollectionName =
  | "EXPEDITION"
  | "MAD_SCIENTISTS"
  | "BAD_KIDS"
  | "SLOTH"
  | "PIGEON"
  | "MOO";

// TODO: use actual neutron addresses
export const COLLECTION: CollectionConfig = {
  // -------------------------------------------------------------------------------------
  MAINNET: {
    EXPEDITION: {
      STARGAZE:
        "stars16srrs6zyl60n2avmp5hlkrc4k37q8spyzjtza7fhtpjchdjumxpq0rrnqm",
      NEUTRON:
        "neutron16srrs6zyl60n2avmp5hlkrc4k37q8spyzjtza7fhtpjchdjumxpqctks3q",
    },

    MAD_SCIENTISTS: {
      STARGAZE:
        "stars1v8avajk64z7pppeu45ce6vv8wuxmwacdff484lqvv0vnka0cwgdqdk64sf",
      NEUTRON:
        "neutron1v8avajk64z7pppeu45ce6vv8wuxmwacdff484lqvv0vnka0cwgdq670kpj",
    },

    BAD_KIDS: {
      STARGAZE:
        "stars19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseues57n420",
      NEUTRON:
        "neutron19jq6mj84cnt9p7sagjxqf8hxtczwc8wlpuwe4sh62w45aheseuesrkxkm5",
    },

    SLOTH: {
      STARGAZE:
        "stars10n0m58ztlr9wvwkgjuek2m2k0dn5pgrhfw9eahg9p8e5qtvn964suc995j",
      NEUTRON:
        "neutron10n0m58ztlr9wvwkgjuek2m2k0dn5pgrhfw9eahg9p8e5qtvn964stssx9f",
    },

    PIGEON: {
      STARGAZE:
        "stars12c9nrpkqrfmdvrx4ex8d6qfs8rwrnclsk5jtk4r6u4gy9mjl97js626dtp",
      NEUTRON:
        "neutron12c9nrpkqrfmdvrx4ex8d6qfs8rwrnclsk5jtk4r6u4gy9mjl97jsdz0w66",
    },
  },

  // -------------------------------------------------------------------------------------
  TESTNET: {
    BAD_KIDS: {
      STARGAZE:
        "stars1pfv9a78y99ek6nezc9zq5f2u0kcwgh0ty0y3dh6wn2lgk27qs9hqefncau",
      NEUTRON:
        "neutron1e4fqdkym8e0ac4humywngx0lqw3ccq4nx07ene4qup5dr0plrjcq8nz6dn",
    },

    SLOTH: {
      STARGAZE:
        "stars1ey7dgnxfcjlz4pgn47k4ygjl3q445vvnr2x6qttnew7wlrkyqeysutm2cv",
      NEUTRON:
        "neutron1lgrh7ukjwlrsp3dmjdh36pl7z5ldmd94wjs5q2xapgttk7lzdfgsyd8wye",
    },

    PIGEON: {
      STARGAZE:
        "stars1qrghctped3a7jcklqxg92dn8lvw88adrduwx3h50pmmcgcwl82xsu84lnw",
      NEUTRON:
        "neutron1skwhr5eu3hc47qflclg88ddnxp0fh643h4huwkcgc5wtr5j2qj5qxdm45p",
    },

    // MOO: {
    //   STARGAZE:
    //     "stars1w4rk4zpme2axwr2r6g8qgptcvpgktmyt2mh94vtyum8yt25guvdqw6rwad",
    //   NEUTRON:
    //     "neutron1w4rk4zpme2axwr2r6g8qgptcvpgktmyt2mh94vtyum8yt25guvdqejkdvk",
    // },
  },
};
