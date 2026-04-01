"""Generate production lists as CSV files."""

import csv
import os

MEMBER_LABELS = {
    "frame_top": "Bovendorpel", "frame_bottom": "Onderdorpel",
    "frame_left": "Stijl links", "frame_right": "Stijl rechts",
    "divider_h": "Tussendorpel", "divider_v": "Tussenstijl",
    "sash_top": "Raamhout boven", "sash_bottom": "Raamhout onder",
    "sash_left": "Raamhout links", "sash_right": "Raamhout rechts",
}

GASKET_LABELS = {
    "glazing_inner": "Binnenrubber beglazing", "glazing_outer": "Buitenrubber beglazing",
    "sash_seal": "Vleugelafdichting", "frame_seal": "Kozijnafdichting",
}


def generate_production_csv(production_data_list: list, output_dir: str):
    """Generate CSV files for each production list type."""
    os.makedirs(output_dir, exist_ok=True)

    all_cut = []
    all_glass = []
    all_hw = []
    all_gasket = []
    all_panel = []
    all_bom = []

    for prod in production_data_list:
        mark = prod.get("kozijnMark", "?")
        for item in prod.get("cutList", []):
            item["_mark"] = mark
            all_cut.append(item)
        for item in prod.get("glassList", []):
            item["_mark"] = mark
            all_glass.append(item)
        for item in prod.get("hardwareList", []):
            item["_mark"] = mark
            all_hw.append(item)
        for item in prod.get("gasketList", []):
            item["_mark"] = mark
            all_gasket.append(item)
        for item in prod.get("panelList", []):
            item["_mark"] = mark
            all_panel.append(item)
        for item in prod.get("bom", []):
            item["_mark"] = mark
            all_bom.append(item)

    # Kortlijst
    with open(os.path.join(output_dir, "kortlijst.csv"), "w", newline="", encoding="utf-8-sig") as f:
        w = csv.writer(f, delimiter=";")
        w.writerow(["Kozijn", "Pos", "Onderdeel", "Profiel", "Materiaal",
                     "Netto_mm", "Bruto_mm", "Hoek_L", "Hoek_R", "Aantal"])
        for item in all_cut:
            member = MEMBER_LABELS.get(item.get("memberType", ""), item.get("memberType", ""))
            w.writerow([item["_mark"], item.get("pieceId", ""), member,
                        item.get("profileName", ""), item.get("material", ""),
                        round(item.get("netLengthMm", 0)),
                        round(item.get("grossLengthMm", 0)),
                        round(item.get("miterLeftDeg", 90)),
                        round(item.get("miterRightDeg", 90)),
                        item.get("quantity", 1)])

    # Glaslijst
    with open(os.path.join(output_dir, "glaslijst.csv"), "w", newline="", encoding="utf-8-sig") as f:
        w = csv.writer(f, delimiter=";")
        w.writerow(["Kozijn", "Pos", "Glastype", "Breedte_mm", "Hoogte_mm",
                     "Dikte_mm", "Ug", "Opp_m2", "Aantal"])
        for item in all_glass:
            w.writerow([item["_mark"], item.get("pieceId", ""), item.get("glassType", ""),
                        round(item.get("widthMm", 0)), round(item.get("heightMm", 0)),
                        round(item.get("thicknessMm", 0)), round(item.get("ugValue", 0), 1),
                        round(item.get("areaM2", 0), 2), item.get("quantity", 1)])

    # Beslaglijst
    with open(os.path.join(output_dir, "beslaglijst.csv"), "w", newline="", encoding="utf-8-sig") as f:
        w = csv.writer(f, delimiter=";")
        w.writerow(["Kozijn", "Cel", "Component", "Omschrijving", "Aantal"])
        for item in all_hw:
            w.writerow([item["_mark"], item.get("cellIndex", 0) + 1,
                        item.get("component", ""), item.get("description", ""),
                        item.get("quantity", 1)])

    # Rubberlijst
    with open(os.path.join(output_dir, "rubberlijst.csv"), "w", newline="", encoding="utf-8-sig") as f:
        w = csv.writer(f, delimiter=";")
        w.writerow(["Kozijn", "Type", "Lengte_mm", "Aantal"])
        for item in all_gasket:
            label = GASKET_LABELS.get(item.get("gasketType", ""), item.get("gasketType", ""))
            w.writerow([item["_mark"], label, round(item.get("lengthMm", 0)),
                        item.get("quantity", 1)])

    # Stuklijst
    with open(os.path.join(output_dir, "stuklijst.csv"), "w", newline="", encoding="utf-8-sig") as f:
        w = csv.writer(f, delimiter=";")
        w.writerow(["Kozijn", "Categorie", "Omschrijving", "Eenheid", "Hoeveelheid"])
        for item in all_bom:
            w.writerow([item["_mark"], item.get("category", ""), item.get("description", ""),
                        item.get("unit", ""), round(item.get("quantity", 0), 2)])

    return output_dir
