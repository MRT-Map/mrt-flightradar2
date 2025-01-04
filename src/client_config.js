import * as L from "leaflet";


class CustomTileLayer extends L.TileLayer {
    getTileUrl(coords) {
        const Zcoord = 2 ** (8 - coords.z);
        const Xcoord = coords.x * 1;
        const Ycoord = coords.y * -1;
        const group = {
            x: Math.floor((Xcoord * Zcoord) / 32),
            y: Math.floor((Ycoord * Zcoord) / 32),
        };
        const numberInGroup = {
            x: Math.floor(Xcoord * Zcoord),
            y: Math.floor(Ycoord * Zcoord),
        };
        let zzz = "";
        for (let i = 8; i > coords.z; i--) {
            zzz += "z";
        }
        if (zzz.length !== 0) zzz += "_";
        return `https://dynmap.minecartrapidtransit.net/main/tiles/new/flat/${group.x}_${group.y}/${zzz}${numberInGroup.x}_${numberInGroup.y}.png`;
    }
}

export const tileLayer = new CustomTileLayer("", {
    maxZoom: 8,
    id: "map",
    tileSize: 128,
    zoomOffset: 0,
    noWrap: true,
    bounds: [
        [-900, -900],
        [900, 900],
    ],
    attribution: "Minecart Rapid Transit",
});

export const altitudeColours = [
    [50, "#aaaaaa"],
    [100, "#aaaa00"],
    [150, "#00aa00"],
    [250, "#00aaaa"],
    [350, "#0000aa"],
    [450, "#aa00aa"],
    [550, "#aa0000"],
    [650, "#000000"],
];

export function world2map([x, y]) {
    return [y / -64 - 0.5, x / 64];
}

export function world2map3([x, y, z]) {
    return [y / -64 - 0.5, x / 64, z];
}