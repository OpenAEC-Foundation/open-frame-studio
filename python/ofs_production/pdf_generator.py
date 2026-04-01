"""Generate production lists as a multi-section PDF document."""

from reportlab.lib.pagesizes import A4, landscape
from reportlab.lib import colors
from reportlab.lib.units import mm
from reportlab.platypus import SimpleDocTemplate, Table, TableStyle, Paragraph, Spacer, PageBreak
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_CENTER, TA_LEFT

AMBER = colors.HexColor("#D97706")
DEEP_FORGE = colors.HexColor("#36363E")
LIGHT_BG = colors.HexColor("#F9FAFB")

MEMBER_LABELS = {
    "frame_top": "Bovendorpel",
    "frame_bottom": "Onderdorpel",
    "frame_left": "Stijl links",
    "frame_right": "Stijl rechts",
    "divider_h": "Tussendorpel",
    "divider_v": "Tussenstijl",
    "sash_top": "Raamhout boven",
    "sash_bottom": "Raamhout onder",
    "sash_left": "Raamhout links",
    "sash_right": "Raamhout rechts",
}

GASKET_LABELS = {
    "glazing_inner": "Binnenrubber beglazing",
    "glazing_outer": "Buitenrubber beglazing",
    "sash_seal": "Vleugelafdichting",
    "frame_seal": "Kozijnafdichting",
}


def generate_production_pdf(production_data_list: list, output_path: str):
    """Generate a PDF with all production lists for one or more kozijnen."""
    doc = SimpleDocTemplate(
        output_path,
        pagesize=landscape(A4),
        leftMargin=12 * mm,
        rightMargin=12 * mm,
        topMargin=12 * mm,
        bottomMargin=12 * mm,
    )

    styles = getSampleStyleSheet()
    title_style = ParagraphStyle("ProdTitle", parent=styles["Title"],
                                  fontSize=16, textColor=DEEP_FORGE, spaceAfter=4 * mm)
    section_style = ParagraphStyle("ProdSection", parent=styles["Heading2"],
                                    fontSize=12, textColor=AMBER, spaceAfter=3 * mm,
                                    spaceBefore=6 * mm)
    normal = ParagraphStyle("ProdNormal", parent=styles["Normal"], fontSize=8)

    story = []

    for prod in production_data_list:
        mark = prod.get("kozijnMark", "?")
        name = prod.get("kozijnName", "")
        story.append(Paragraph(f"Productiestaten — {mark} {name}", title_style))

        # -- Kortlijst --
        cut_list = prod.get("cutList", [])
        if cut_list:
            story.append(Paragraph("Kortlijst (Afkortlijst)", section_style))
            headers = ["Pos.", "Onderdeel", "Profiel", "Materiaal", "Netto (mm)",
                        "Bruto (mm)", "Hoek L", "Hoek R", "Aantal"]
            rows = [headers]
            for item in cut_list:
                member = MEMBER_LABELS.get(item.get("memberType", ""), item.get("memberType", ""))
                rows.append([
                    item.get("pieceId", ""),
                    member,
                    item.get("profileName", ""),
                    item.get("material", ""),
                    f'{item.get("netLengthMm", 0):.0f}',
                    f'{item.get("grossLengthMm", 0):.0f}',
                    f'{item.get("miterLeftDeg", 90):.0f}\u00b0',
                    f'{item.get("miterRightDeg", 90):.0f}\u00b0',
                    str(item.get("quantity", 1)),
                ])
            story.append(_make_table(rows))

        # -- Glaslijst --
        glass_list = prod.get("glassList", [])
        if glass_list:
            story.append(Paragraph("Glaslijst", section_style))
            headers = ["Pos.", "Glastype", "Breedte (mm)", "Hoogte (mm)",
                        "Dikte (mm)", "Ug", "Opp. (m\u00b2)", "Aantal"]
            rows = [headers]
            for item in glass_list:
                rows.append([
                    item.get("pieceId", ""),
                    item.get("glassType", ""),
                    f'{item.get("widthMm", 0):.0f}',
                    f'{item.get("heightMm", 0):.0f}',
                    f'{item.get("thicknessMm", 0):.0f}',
                    f'{item.get("ugValue", 0):.1f}',
                    f'{item.get("areaM2", 0):.2f}',
                    str(item.get("quantity", 1)),
                ])
            story.append(_make_table(rows))

        # -- Beslaglijst --
        hw_list = prod.get("hardwareList", [])
        if hw_list:
            story.append(Paragraph("Beslaglijst", section_style))
            headers = ["Cel", "Component", "Omschrijving", "Aantal"]
            rows = [headers]
            for item in hw_list:
                rows.append([
                    str(item.get("cellIndex", 0) + 1),
                    item.get("component", ""),
                    item.get("description", ""),
                    str(item.get("quantity", 1)),
                ])
            story.append(_make_table(rows))

        # -- Rubberlijst --
        gasket_list = prod.get("gasketList", [])
        if gasket_list:
            story.append(Paragraph("Rubberlijst", section_style))
            headers = ["Type", "Lengte (mm)", "Aantal"]
            rows = [headers]
            for item in gasket_list:
                label = GASKET_LABELS.get(item.get("gasketType", ""), item.get("gasketType", ""))
                rows.append([
                    label,
                    f'{item.get("lengthMm", 0):.0f}',
                    str(item.get("quantity", 1)),
                ])
            story.append(_make_table(rows))

        # -- Paneellijst --
        panel_list = prod.get("panelList", [])
        if panel_list:
            story.append(Paragraph("Paneellijst", section_style))
            headers = ["Pos.", "Breedte (mm)", "Hoogte (mm)", "Type", "Aantal"]
            rows = [headers]
            for item in panel_list:
                rows.append([
                    item.get("pieceId", ""),
                    f'{item.get("widthMm", 0):.0f}',
                    f'{item.get("heightMm", 0):.0f}',
                    item.get("panelType", ""),
                    str(item.get("quantity", 1)),
                ])
            story.append(_make_table(rows))

        # -- Stuklijst (BOM) --
        bom = prod.get("bom", [])
        if bom:
            story.append(Paragraph("Stuklijst (BOM)", section_style))
            headers = ["Categorie", "Omschrijving", "Eenheid", "Hoeveelheid"]
            rows = [headers]
            for item in bom:
                rows.append([
                    item.get("category", ""),
                    item.get("description", ""),
                    item.get("unit", ""),
                    f'{item.get("quantity", 0):.2f}',
                ])
            story.append(_make_table(rows))

        story.append(PageBreak())

    doc.build(story)


def _make_table(rows):
    """Create a styled ReportLab table."""
    col_count = len(rows[0]) if rows else 0
    table = Table(rows, repeatRows=1)
    style = [
        ("BACKGROUND", (0, 0), (-1, 0), DEEP_FORGE),
        ("TEXTCOLOR", (0, 0), (-1, 0), colors.white),
        ("FONTSIZE", (0, 0), (-1, -1), 8),
        ("FONTNAME", (0, 0), (-1, 0), "Helvetica-Bold"),
        ("ALIGN", (0, 0), (-1, 0), "CENTER"),
        ("BOTTOMPADDING", (0, 0), (-1, 0), 4),
        ("TOPPADDING", (0, 0), (-1, 0), 4),
        ("GRID", (0, 0), (-1, -1), 0.5, colors.HexColor("#E5E7EB")),
        ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
        ("TOPPADDING", (0, 1), (-1, -1), 2),
        ("BOTTOMPADDING", (0, 1), (-1, -1), 2),
    ]
    # Alternating row colors
    for i in range(1, len(rows)):
        if i % 2 == 0:
            style.append(("BACKGROUND", (0, i), (-1, i), LIGHT_BG))
    # Right-align numeric columns (skip first 2 columns)
    for col in range(2, col_count):
        style.append(("ALIGN", (col, 1), (col, -1), "RIGHT"))

    table.setStyle(TableStyle(style))
    return table
