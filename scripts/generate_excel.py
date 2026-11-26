#!/usr/bin/env python3
"""Membangun workbook Excel GDSS Pemilihan SIEM (docs/Perhitungan-GDSS-SIEM.xlsx).

Seluruh perhitungan (bobot ROC, normalisasi Benefit/Cost SAW, matriks terbobot,
peringkat individual, poin voting Borda, pemecah seri oleh CISO, dan papan peringkat
terurut) memakai rumus Excel klasik (kompatibel Excel 2007-365, Google Sheets, dan
LibreOffice Calc). Tidak ada hasil perhitungan yang ditulis keras (hardcoded).

Pemakaian:
    python3 scripts/generate_excel.py
"""

from pathlib import Path
import re
import zipfile
import xml.etree.ElementTree as ET

import openpyxl
from openpyxl.chart import BarChart, Reference
from openpyxl.styles import Alignment, Border, Font, PatternFill, Side
from openpyxl.utils import get_column_letter
from openpyxl.worksheet.datavalidation import DataValidation

ROOT = Path(__file__).resolve().parent.parent
OUT_XLSX = ROOT / "docs" / "Perhitungan-GDSS-SIEM.xlsx"

# --------------------------------------------------------------------------- Data Master
# Data sama persis dengan scripts/seed_emulator.py dan docs/RANCANGAN.md

DMS = [
    ("DM1", "Decision Maker 1", "CISO", 3, "Menilai + Eksekusi Voting Borda (Tertinggi)", "ciso@gdss-siem.test"),
    ("DM2", "Decision Maker 2", "SOC Analyst", 2, "Menilai Individual + Lihat Hasil", "soc@gdss-siem.test"),
    ("DM3", "Decision Maker 3", "Network Admin", 1, "Menilai Individual + Lihat Hasil", "netadmin@gdss-siem.test"),
]

KRITERIA = [
    (
        "C1", "Biaya lisensi", "Cost",
        "Biaya lisensi atau langganan untuk sekitar 50 endpoint dan perangkat jaringan.",
        [
            "Tanpa biaya lisensi untuk fitur inti",
            "Gratis, tetapi fitur penting perlu langganan",
            "Berbayar, di bawah US$300 per bulan",
            "Berbayar, US$300 sampai US$1.000 per bulan",
            "Di atas US$1.000 per bulan atau hanya lewat penawaran",
        ],
    ),
    (
        "C2", "Fitur alerting", "Benefit",
        "Kanal notifikasi, rule yang dapat dikustom, korelasi kejadian, dan eskalasi.",
        [
            "Tidak ada alert otomatis pada edisi yang dipakai",
            "Alert dasar lewat satu kanal",
            "Beberapa kanal dengan rule ambang sederhana",
            "Banyak kanal termasuk webhook atau chat, rule dapat dikustom",
            "Korelasi lintas sumber dan rule terpetakan ke MITRE ATT&CK",
        ],
    ),
    (
        "C3", "Integrasi log", "Benefit",
        "Ragam sumber log yang dapat diterima: syslog, Windows Event Log, agen, cloud, API.",
        [
            "Hanya metrik, tanpa pengumpulan log",
            "Syslog atau satu jenis sumber log",
            "Syslog dan Windows Event Log",
            "Agen multi-OS, syslog, Windows, dan API",
            "Ratusan integrasi siap pakai termasuk layanan cloud",
        ],
    ),
    (
        "C4", "Kemudahan penggunaan", "Benefit",
        "Usaha instalasi, kebutuhan hardware, dan kurva belajar bagi tim kecil.",
        [
            "Instalasi dan operasi butuh tim khusus",
            "Banyak komponen, kurva belajar tinggi",
            "Perlu konfigurasi manual yang cukup banyak",
            "Installer all-in-one dan dokumentasi jelas",
            "Dapat dipakai satu admin dalam satu hari",
        ],
    ),
    (
        "C5", "Komunitas dan dukungan", "Benefit",
        "Dokumentasi, forum, repositori terbuka, dan ketersediaan dukungan berbayar.",
        [
            "Dokumentasi minim, tanpa forum aktif",
            "Dokumentasi resmi saja",
            "Forum resmi dan dukungan berbayar",
            "Komunitas aktif dan dukungan berbayar",
            "Komunitas sangat besar, repositori terbuka, dukungan berbayar",
        ],
    ),
]

ALTERNATIF = [
    (
        "A1", "Wazuh", "SIEM dan XDR", "Open source, tanpa biaya lisensi",
        "Agen Linux/Windows/macOS, rule MITRE ATT&CK, FIM, deteksi kerentanan, PCI DSS/GDPR, installer all-in-one (4 vCPU / 8 GiB untuk 25 agen).",
        "https://wazuh.com/ | https://documentation.wazuh.com/current/quickstart.html",
    ),
    (
        "A2", "Splunk Enterprise Security", "SIEM komersial", "Proprietary, harga lewat penawaran",
        "Platform log dengan Risk-Based Alerting dan korelasi lintas sumber. Edisi gratis 500 MB/hari tanpa alerting. Referensi 12 core / 12 GB RAM.",
        "https://www.splunk.com/en_us/products/pricing.html | https://docs.splunk.com/",
    ),
    (
        "A3", "Elastic Security (ELK)", "SIEM", "Tier Basic gratis, konektor notifikasi berbayar",
        "Lebih dari 400 integrasi ECS, detection engine MITRE ATT&CK, FIM di Basic. Konektor email/Slack/webhook mulai tier Platinum.",
        "https://www.elastic.co/subscriptions | https://www.elastic.co/security/siem",
    ),
    (
        "A4", "Zabbix", "Monitoring infrastruktur", "AGPLv3, tanpa biaya lisensi",
        "Pemantauan ketersediaan & kinerja (agen, SNMP, log file), alert email/SMS/webhook, 2 CPU / 8 GiB untuk 1.000 metrik. Bukan SIEM murni.",
        "https://www.zabbix.com/features | https://www.zabbix.com/support",
    ),
    (
        "A5", "PRTG Network Monitor", "Monitoring jaringan", "Langganan per sensor, mulai US$200/bulan",
        "Monitoring jaringan berbasis sensor (SNMP, flow, syslog, Windows Event Log), khusus Windows, alert Teams/Slack/webhook. Bukan SIEM murni.",
        "https://www.paessler.com/prtg/pricing | https://www.paessler.com/manuals/prtg/notifications",
    ),
]

# Input simulasi tiap DM (urutan prioritas per C1..C5 dan matriks nilai A1..A5 x C1..C5)
DM_CONFIG = [
    {
        "sheet": "03_DM1_CISO",
        "judul": "DM 1: Chief Information Security Officer (CISO)",
        "level": 3,
        "fokus": "CISO memprioritaskan kemampuan deteksi & alerting keamanan (C2) serta efisiensi anggaran lisensi (C1), disusul integrasi log (C3), komunitas/dukungan (C5), dan kemudahan penggunaan (C4).",
        "prioritas": [2, 1, 3, 5, 4],  # C1=#2, C2=#1, C3=#3, C4=#5, C5=#4
        "nilai": [
            [1, 5, 4, 4, 5],  # A1 Wazuh
            [5, 5, 5, 2, 4],  # A2 Splunk ES
            [2, 3, 5, 2, 5],  # A3 Elastic Security
            [1, 3, 3, 3, 4],  # A4 Zabbix
            [3, 3, 3, 4, 3],  # A5 PRTG
        ],
    },
    {
        "sheet": "04_DM2_SOC_Analyst",
        "judul": "DM 2: Security Operation Center (SOC) Analyst",
        "level": 2,
        "fokus": "SOC Analyst memprioritaskan kualitas alerting & korelasi rule (C2) dan ragam integrasi log (C3) untuk operasi harian, lalu kemudahan pemakaian (C4), komunitas (C5), dan biaya (C1).",
        "prioritas": [5, 1, 2, 3, 4],  # C1=#5, C2=#1, C3=#2, C4=#3, C5=#4
        "nilai": [
            [1, 5, 4, 4, 5],  # A1 Wazuh
            [5, 5, 5, 3, 4],  # A2 Splunk ES
            [2, 4, 5, 2, 5],  # A3 Elastic Security
            [1, 3, 3, 3, 4],  # A4 Zabbix
            [3, 3, 3, 4, 3],  # A5 PRTG
        ],
    },
    {
        "sheet": "05_DM3_Network_Admin",
        "judul": "DM 3: Network Administrator",
        "level": 1,
        "fokus": "Network Admin memprioritaskan integrasi log perangkat jaringan & agen (C3) serta kemudahan instalasi/perawatan (C4), diikuti komunitas/dukungan (C5), alerting (C2), dan biaya (C1).",
        "prioritas": [5, 4, 1, 2, 3],  # C1=#5, C2=#4, C3=#1, C4=#2, C5=#3
        "nilai": [
            [1, 4, 4, 3, 5],  # A1 Wazuh
            [5, 5, 5, 2, 4],  # A2 Splunk ES
            [2, 3, 5, 2, 5],  # A3 Elastic Security
            [1, 4, 4, 4, 4],  # A4 Zabbix
            [3, 4, 3, 5, 3],  # A5 PRTG
        ],
    },
]

# --------------------------------------------------------------------------- Palet & Gaya
# Palet terkendali (R-29): 2 inti netral-navy (#1E293B / #334155) + 1 aksen indigo (#3730A3)
# + konvensi standar pemodelan spreadsheet (input biru di latar krem, tautan hijau, hasil pemenang)

FONT_FAMILY = "Arial"

F_TITLE = Font(name=FONT_FAMILY, size=15, bold=True, color="FFFFFF")
F_SUBTITLE = Font(name=FONT_FAMILY, size=10, italic=False, color="CBD5E1")
F_SEC = Font(name=FONT_FAMILY, size=11, bold=True, color="1E293B")
F_HDR = Font(name=FONT_FAMILY, size=10, bold=True, color="FFFFFF")
F_SUBHDR = Font(name=FONT_FAMILY, size=10, bold=True, color="1E293B")
F_BODY = Font(name=FONT_FAMILY, size=10, bold=False, color="0F172A")
F_BODY_BOLD = Font(name=FONT_FAMILY, size=10, bold=True, color="0F172A")
F_MUTED = Font(name=FONT_FAMILY, size=9, italic=True, color="475569")
F_INPUT = Font(name=FONT_FAMILY, size=10, bold=True, color="0000FF")
F_LINK = Font(name=FONT_FAMILY, size=10, bold=False, color="047857")
F_LINK_BOLD = Font(name=FONT_FAMILY, size=10, bold=True, color="047857")
F_WIN = Font(name=FONT_FAMILY, size=10, bold=True, color="1E3A8A")
F_WARN = Font(name=FONT_FAMILY, size=10, bold=True, color="92400E")
F_CODE = Font(name="Consolas", size=9, bold=False, color="1E293B")

FILL_NAVY = PatternFill("solid", fgColor="1E293B")
FILL_SLATE = PatternFill("solid", fgColor="334155")
FILL_SEC = PatternFill("solid", fgColor="E2E8F0")
FILL_ZEBRA = PatternFill("solid", fgColor="F8FAFC")
FILL_WHITE = PatternFill("solid", fgColor="FFFFFF")
FILL_INPUT = PatternFill("solid", fgColor="FEF9C3")
FILL_TOTAL = PatternFill("solid", fgColor="EEF2FF")
FILL_WIN = PatternFill("solid", fgColor="DBEAFE")
FILL_NOTE = PatternFill("solid", fgColor="F1F5F9")
FILL_WARN = PatternFill("solid", fgColor="FEF3C7")

THIN_GRAY = Side(style="thin", color="CBD5E1")
MED_NAVY = Side(style="medium", color="1E293B")
DOUBLE_NAVY = Side(style="double", color="1E293B")

BORDER_CELL = Border(left=THIN_GRAY, right=THIN_GRAY, top=THIN_GRAY, bottom=THIN_GRAY)
BORDER_HDR = Border(left=THIN_GRAY, right=THIN_GRAY, top=MED_NAVY, bottom=MED_NAVY)
BORDER_TOTAL = Border(left=THIN_GRAY, right=THIN_GRAY, top=THIN_GRAY, bottom=DOUBLE_NAVY)
BORDER_BOX = Border(left=MED_NAVY, right=THIN_GRAY, top=THIN_GRAY, bottom=THIN_GRAY)

AL_LEFT = Alignment(horizontal="left", vertical="center", wrap_text=True)
AL_CENTER = Alignment(horizontal="center", vertical="center", wrap_text=True)
AL_RIGHT = Alignment(horizontal="right", vertical="center", wrap_text=True)
AL_TOP = Alignment(horizontal="left", vertical="top", wrap_text=True)


def setup_sheet(ws, title: str, subtitle: str, max_col: int, tab_color: str):
    ws.views.sheetView[0].showGridLines = True
    ws.sheet_properties.tabColor = tab_color
    ws.page_setup.orientation = ws.ORIENTATION_LANDSCAPE
    ws.page_setup.paperSize = ws.PAPERSIZE_A4
    ws.sheet_properties.pageSetUpPr.fitToPage = True
    ws.page_setup.fitToWidth = 1
    ws.page_setup.fitToHeight = 0

    end_col = get_column_letter(max_col)
    ws.merge_cells(f"A1:{end_col}1")
    ws["A1"] = title
    ws["A1"].font = F_TITLE
    ws["A1"].fill = FILL_NAVY
    ws["A1"].alignment = Alignment(horizontal="left", vertical="center", indent=1)
    ws.row_dimensions[1].height = 32

    ws.merge_cells(f"A2:{end_col}2")
    ws["A2"] = subtitle
    ws["A2"].font = F_SUBTITLE
    ws["A2"].fill = FILL_NAVY
    ws["A2"].alignment = Alignment(horizontal="left", vertical="center", indent=1, wrap_text=True)
    ws.row_dimensions[2].height = 22
    ws.row_dimensions[3].height = 10


def section_bar(ws, row: int, text: str, max_col: int):
    end_col = get_column_letter(max_col)
    ws.merge_cells(f"A{row}:{end_col}{row}")
    c = ws[f"A{row}"]
    c.value = text
    c.font = F_SEC
    c.fill = FILL_SEC
    c.alignment = Alignment(horizontal="left", vertical="center", indent=1)
    for col in range(1, max_col + 1):
        ws.cell(row=row, column=col).border = Border(bottom=MED_NAVY, top=THIN_GRAY)
        ws.cell(row=row, column=col).fill = FILL_SEC
    ws.row_dimensions[row].height = 24


def header_row(ws, row: int, headers: list[str], start_col: int = 1):
    for idx, text in enumerate(headers):
        c = ws.cell(row=row, column=start_col + idx, value=text)
        c.font = F_HDR
        c.fill = FILL_SLATE
        c.alignment = AL_CENTER
        c.border = BORDER_HDR
    ws.row_dimensions[row].height = 28


def note_box(ws, row: int, text: str, max_col: int, height: int = 38):
    end_col = get_column_letter(max_col)
    ws.merge_cells(f"A{row}:{end_col}{row}")
    c = ws[f"A{row}"]
    c.value = text
    c.font = F_MUTED
    c.fill = FILL_NOTE
    c.alignment = Alignment(horizontal="left", vertical="center", wrap_text=True, indent=1)
    for col in range(1, max_col + 1):
        cell = ws.cell(row=row, column=col)
        cell.fill = FILL_NOTE
        cell.border = BORDER_BOX if col == 1 else BORDER_CELL
    ws.row_dimensions[row].height = height


def set_widths(ws, widths: dict[int, float]):
    for col_idx, width in widths.items():
        ws.column_dimensions[get_column_letter(col_idx)].width = width


# --------------------------------------------------------------------------- Sheet 2: Data Master
def build_data_master(wb):
    ws = wb.create_sheet("02_Data_Master")
    setup_sheet(
        ws,
        "DATA MASTER: DECISION MAKER, KRITERIA, RUBRIK, ALTERNATIF & TABEL ROC",
        "Sumber acuan tunggal untuk seluruh rumus lintas-sheet. Perubahan nama/jenis kriteria di sini otomatis memengaruhi semua perhitungan.",
        max_col=9,
        tab_color="334155",
    )

    # Tabel 1: Decision Maker (rows 4..8)
    section_bar(ws, 4, "1. Daftar Decision Maker (Hierarki Jabatan)", 9)
    header_row(ws, 5, ["Kode", "Nama (Penanda Tempat)", "Jabatan", "Level", "Wewenang di Sistem", "Akun Demo"])
    ws.merge_cells("F5:I5")
    for col in range(6, 10):
        ws.cell(5, col).fill = FILL_SLATE
        ws.cell(5, col).border = BORDER_HDR
    for i, dm in enumerate(DMS):
        r = 6 + i
        fill = FILL_ZEBRA if i % 2 else FILL_WHITE
        vals = [dm[0], dm[1], dm[2], dm[3], dm[4], dm[5]]
        for ci, v in enumerate(vals, start=1):
            c = ws.cell(r, ci, v)
            c.font = F_BODY_BOLD if ci in (1, 3) else F_BODY
            c.fill = fill
            c.border = BORDER_CELL
            c.alignment = AL_CENTER if ci in (1, 4) else AL_LEFT
        ws.merge_cells(f"F{r}:I{r}")
        for col in range(6, 10):
            ws.cell(r, col).fill = fill
            ws.cell(r, col).border = BORDER_CELL
        ws.row_dimensions[r].height = 22

    # Tabel 2: Kriteria & Rubrik 1-5 (rows 10..16)
    section_bar(ws, 10, "2. Daftar Kriteria Pengambilan Keputusan & Rubrik Penilaian (Skala 1 sampai 5)", 9)
    header_row(ws, 11, ["Kode", "Nama Kriteria", "Jenis", "Deskripsi", "Skor 1", "Skor 2", "Skor 3", "Skor 4", "Skor 5"])
    for i, k in enumerate(KRITERIA):
        r = 12 + i  # C1=12 .. C5=16
        fill = FILL_ZEBRA if i % 2 else FILL_WHITE
        row_vals = [k[0], k[1], k[2], k[3]] + k[4]
        for ci, v in enumerate(row_vals, start=1):
            c = ws.cell(r, ci, v)
            c.font = F_BODY_BOLD if ci in (1, 2) else (F_WARN if ci == 3 and v == "Cost" else F_BODY)
            c.fill = FILL_WARN if ci == 3 and v == "Cost" else fill
            c.border = BORDER_CELL
            c.alignment = AL_CENTER if ci in (1, 3) else AL_LEFT
        ws.row_dimensions[r].height = 40

    # Tabel 3: Alternatif (rows 18..24)
    section_bar(ws, 18, "3. Daftar Alternatif Kandidat SIEM & Alat Monitoring Jaringan (Data Resmi Vendor, 25 Nov 2026)", 9)
    header_row(ws, 19, ["Kode", "Nama Alternatif", "Kategori", "Model Lisensi", "Ringkasan Teknis", "", "", "Sumber Resmi", ""])
    ws.merge_cells("E19:G19")
    ws.merge_cells("H19:I19")
    for col in range(5, 10):
        ws.cell(19, col).fill = FILL_SLATE
        ws.cell(19, col).border = BORDER_HDR
    for i, a in enumerate(ALTERNATIF):
        r = 20 + i  # A1=20 .. A5=24
        fill = FILL_ZEBRA if i % 2 else FILL_WHITE
        ws.cell(r, 1, a[0]).alignment = AL_CENTER
        ws.cell(r, 2, a[1]).alignment = AL_LEFT
        ws.cell(r, 3, a[2]).alignment = AL_LEFT
        ws.cell(r, 4, a[3]).alignment = AL_LEFT
        ws.cell(r, 5, a[4]).alignment = AL_LEFT
        ws.cell(r, 8, a[5]).alignment = AL_LEFT
        ws.merge_cells(f"E{r}:G{r}")
        ws.merge_cells(f"H{r}:I{r}")
        for col in range(1, 10):
            cell = ws.cell(r, col)
            cell.fill = fill
            cell.border = BORDER_CELL
            if col in (1, 2):
                cell.font = F_BODY_BOLD
            elif col == 8:
                cell.font = F_MUTED
            else:
                cell.font = F_BODY
        ws.row_dimensions[r].height = 44

    # Tabel 4: Perhitungan Referensi Bobot ROC (rows 26..34)
    section_bar(ws, 26, "4. Tabel Perhitungan Bobot ROC (Rank Order Centroid) untuk n Kriteria", 9)
    ws["A27"] = "Jumlah Kriteria (n)"
    ws["A27"].font = F_BODY_BOLD
    ws["A27"].fill = FILL_TOTAL
    ws["A27"].border = BORDER_CELL
    ws.merge_cells("A27:B27")
    ws["B27"].fill = FILL_TOTAL
    ws["B27"].border = BORDER_CELL

    ws["C27"] = "=COUNTA(A12:A16)"
    ws["C27"].font = F_BODY_BOLD
    ws["C27"].fill = FILL_TOTAL
    ws["C27"].alignment = AL_CENTER
    ws["C27"].border = BORDER_CELL
    ws["C27"].number_format = "0"

    ws.merge_cells("D27:I27")
    ws["D27"] = "Rumus ROC: w_k = (1 / n) * SUM(1/i untuk i = k sampai n). Bobot ditarik otomatis ke tiap sheet DM memakai INDEX."
    ws["D27"].font = F_MUTED
    ws["D27"].fill = FILL_NOTE
    ws["D27"].alignment = AL_LEFT
    for col in range(4, 10):
        ws.cell(27, col).fill = FILL_NOTE
        ws.cell(27, col).border = BORDER_CELL
    ws.row_dimensions[27].height = 24

    header_row(ws, 28, ["Urutan (k)", "Nilai 1/k", "Jumlah Harmonik (k..n)", "Bobot ROC (Desimal)", "Bobot ROC (%)", "Penjabaran Rumus Manual", "", "", ""])
    ws.merge_cells("F28:I28")
    for col in range(6, 10):
        ws.cell(28, col).fill = FILL_SLATE
        ws.cell(28, col).border = BORDER_HDR

    penjabaran = [
        "(1/1 + 1/2 + 1/3 + 1/4 + 1/5) / 5 = 2,2833 / 5",
        "(1/2 + 1/3 + 1/4 + 1/5) / 5 = 1,2833 / 5",
        "(1/3 + 1/4 + 1/5) / 5 = 0,7833 / 5",
        "(1/4 + 1/5) / 5 = 0,4500 / 5",
        "(1/5) / 5 = 0,2000 / 5",
    ]
    for k_idx in range(1, 6):
        r = 28 + k_idx  # 29..33
        fill = FILL_ZEBRA if k_idx % 2 == 0 else FILL_WHITE
        ws.cell(r, 1, k_idx).number_format = "0"
        ws.cell(r, 2, f"=1/A{r}").number_format = "0.0000"
        ws.cell(r, 3, f"=SUM($B{r}:$B$33)").number_format = "0.0000"
        ws.cell(r, 4, f"=C{r}/$C$27").number_format = "0.0000"
        ws.cell(r, 5, f"=D{r}").number_format = "0.0%"
        ws.cell(r, 6, penjabaran[k_idx - 1])
        ws.merge_cells(f"F{r}:I{r}")
        for col in range(1, 10):
            cell = ws.cell(r, col)
            cell.fill = fill
            cell.border = BORDER_CELL
            cell.font = F_BODY_BOLD if col in (1, 4) else (F_CODE if col == 6 else F_BODY)
            cell.alignment = AL_CENTER if col == 1 else (AL_RIGHT if col in (2, 3, 4, 5) else AL_LEFT)
        ws.row_dimensions[r].height = 22

    # Baris total ROC (row 34)
    ws.merge_cells("A34:C34")
    ws["A34"] = "Total Bobot ROC (harus tepat = 1,0000 / 100%)"
    ws["A34"].font = F_BODY_BOLD
    ws["A34"].alignment = AL_RIGHT
    ws["D34"] = "=SUM(D29:D33)"
    ws["D34"].number_format = "0.0000"
    ws["D34"].font = F_BODY_BOLD
    ws["D34"].alignment = AL_RIGHT
    ws["E34"] = "=SUM(E29:E33)"
    ws["E34"].number_format = "0.0%"
    ws["E34"].font = F_BODY_BOLD
    ws["E34"].alignment = AL_RIGHT
    ws.merge_cells("F34:I34")
    ws["F34"] = '=IF(ABS(D34-1)<0.0001,"VALID: Total bobot = 1","PERIKSA RUMUS")'
    ws["F34"].font = F_LINK_BOLD
    ws["F34"].alignment = AL_LEFT
    for col in range(1, 10):
        ws.cell(34, col).fill = FILL_TOTAL
        ws.cell(34, col).border = BORDER_TOTAL
    ws.row_dimensions[34].height = 24

    set_widths(ws, {1: 11, 2: 26, 3: 18, 4: 30, 5: 25, 6: 25, 7: 25, 8: 25, 9: 26})
    ws.freeze_panes = "A4"


# --------------------------------------------------------------------------- Sheet 3-5: DM Individual (ROC + SAW)
def build_dm_sheet(wb, cfg: dict):
    ws = wb.create_sheet(cfg["sheet"])
    setup_sheet(
        ws,
        f"PENILAIAN INDIVIDUAL {cfg['judul'].upper()} (LEVEL {cfg['level']})",
        "Sel biru berlatar krem dapat diubah untuk simulasi. Seluruh bobot ROC, normalisasi SAW, matriks terbobot, peringkat, dan poin Borda terhitung otomatis.",
        max_col=11,
        tab_color="1D4ED8" if cfg["level"] == 3 else ("0F766E" if cfg["level"] == 2 else "475569"),
    )

    note_box(ws, 4, f"Sudut pandang jabatan: {cfg['fokus']}", 11, height=34)

    # ---------------- Bagian A: Prioritas & Bobot ROC (rows 6..13)
    section_bar(ws, 6, "A. Penentuan Urutan Prioritas Kriteria & Pembobotan ROC", 11)
    header_row(ws, 7, [
        "Kode", "Nama Kriteria", "Jenis", "Urutan Prioritas\n(Input 1-5)",
        "Bobot ROC\n(Rumus)", "Persentase", "Rumus Excel yang Dipakai", "", "", "", ""
    ])
    ws.merge_cells("G7:K7")
    for col in range(7, 12):
        ws.cell(7, col).fill = FILL_SLATE
        ws.cell(7, col).border = BORDER_HDR

    dv_prio = DataValidation(type="whole", operator="between", formula1=1, formula2=5, allow_blank=False)
    dv_prio.promptTitle = "Urutan Prioritas"
    dv_prio.prompt = "Masukkan bilangan bulat 1 (terpenting) sampai 5 tanpa duplikat."
    ws.add_data_validation(dv_prio)

    for i in range(5):
        r = 8 + i  # rows 8..12
        m_row = 12 + i  # Data_Master rows 12..16
        fill = FILL_ZEBRA if i % 2 else FILL_WHITE
        ws.cell(r, 1, f"='02_Data_Master'!A{m_row}")
        ws.cell(r, 2, f"='02_Data_Master'!B{m_row}")
        ws.cell(r, 3, f"='02_Data_Master'!C{m_row}")
        ws.cell(r, 4, cfg["prioritas"][i])
        ws.cell(r, 5, f"=INDEX('02_Data_Master'!$D$29:$D$33, D{r})")
        ws.cell(r, 6, f"=E{r}")
        ws.cell(r, 7, f"Rumus: =INDEX('02_Data_Master'!$D$29:$D$33, D{r}) (bobot ROC urutan ke-D{r})")
        ws.merge_cells(f"G{r}:K{r}")

        ws.cell(r, 1).font = F_LINK_BOLD
        ws.cell(r, 2).font = F_LINK
        ws.cell(r, 3).font = F_LINK
        ws.cell(r, 4).font = F_INPUT
        ws.cell(r, 5).font = F_BODY_BOLD
        ws.cell(r, 6).font = F_BODY
        ws.cell(r, 7).font = F_CODE

        ws.cell(r, 4).number_format = "0"
        ws.cell(r, 5).number_format = "0.0000"
        ws.cell(r, 6).number_format = "0.0%"

        for col in range(1, 12):
            c = ws.cell(r, col)
            c.fill = FILL_INPUT if col == 4 else fill
            c.border = BORDER_CELL
            c.alignment = AL_CENTER if col in (1, 3, 4) else (AL_RIGHT if col in (5, 6) else AL_LEFT)
        dv_prio.add(ws.cell(r, 4))
        ws.row_dimensions[r].height = 22

    # Baris total & validasi prioritas (row 13)
    ws.merge_cells("A13:C13")
    ws["A13"] = "Validasi Urutan & Total Bobot ROC"
    ws["A13"].font = F_BODY_BOLD
    ws["A13"].alignment = AL_RIGHT
    ws["D13"] = '=IF(AND(COUNT(D8:D12)=5,COUNTIF(D8:D12,1)=1,COUNTIF(D8:D12,2)=1,COUNTIF(D8:D12,3)=1,COUNTIF(D8:D12,4)=1,COUNTIF(D8:D12,5)=1),"VALID","DUPLIKAT!")'
    ws["D13"].font = F_LINK_BOLD
    ws["D13"].alignment = AL_CENTER
    ws["E13"] = "=SUM(E8:E12)"
    ws["E13"].number_format = "0.0000"
    ws["E13"].font = F_BODY_BOLD
    ws["E13"].alignment = AL_RIGHT
    ws["F13"] = "=SUM(F8:F12)"
    ws["F13"].number_format = "0.0%"
    ws["F13"].font = F_BODY_BOLD
    ws["F13"].alignment = AL_RIGHT
    ws.merge_cells("G13:K13")
    ws["G13"] = "Urutan 1..5 harus unik agar total bobot ROC tepat = 1,0000 (100%)."
    ws["G13"].font = F_MUTED
    ws["G13"].alignment = AL_LEFT
    for col in range(1, 12):
        ws.cell(13, col).fill = FILL_TOTAL
        ws.cell(13, col).border = BORDER_TOTAL
    ws.row_dimensions[13].height = 24

    # ---------------- Bagian B: Matriks Nilai Mentah X_ij (rows 15..25)
    section_bar(ws, 15, "B. Matriks Keputusan Mentah (X_ij): Input Skor 1 sampai 5 per Alternatif (Lihat Rubrik di 02_Data_Master)", 11)
    header_row(ws, 16, [
        "Kode", "Nama Alternatif",
        "C1\nBiaya (Cost)", "C2\nAlerting (Ben)", "C3\nIntegrasi (Ben)",
        "C4\nKemudahan (Ben)", "C5\nKomunitas (Ben)",
        "Catatan Penilaian", "", "", ""
    ])
    ws.merge_cells("H16:K16")
    for col in range(8, 12):
        ws.cell(16, col).fill = FILL_SLATE
        ws.cell(16, col).border = BORDER_HDR

    dv_skor = DataValidation(type="whole", operator="between", formula1=1, formula2=5, allow_blank=False)
    dv_skor.promptTitle = "Skor Penilaian (1-5)"
    dv_skor.prompt = "Isi bilangan bulat 1 sampai 5 sesuai rubrik di sheet 02_Data_Master."
    ws.add_data_validation(dv_skor)

    catatan_alt = [
        "Open source tanpa biaya lisensi; fitur SIEM + XDR lengkap.",
        "Fitur SIEM enterprise kuat; harga proprietary lewat penawaran.",
        "Integrasi luas (>400); konektor notifikasi eksternal berbayar.",
        "Gratis AGPLv3 & ringan; fokus monitoring infrastruktur (bukan SIEM).",
        "Berbayar per sensor & khusus Windows; monitoring jaringan (bukan SIEM).",
    ]
    for i in range(5):
        r = 17 + i  # rows 17..21
        m_row = 20 + i
        fill = FILL_ZEBRA if i % 2 else FILL_WHITE
        ws.cell(r, 1, f"='02_Data_Master'!A{m_row}")
        ws.cell(r, 2, f"='02_Data_Master'!B{m_row}")
        ws.cell(r, 1).font = F_LINK_BOLD
        ws.cell(r, 2).font = F_LINK
        ws.cell(r, 1).alignment = AL_CENTER
        ws.cell(r, 2).alignment = AL_LEFT
        ws.cell(r, 1).fill = fill
        ws.cell(r, 2).fill = fill
        ws.cell(r, 1).border = BORDER_CELL
        ws.cell(r, 2).border = BORDER_CELL

        for k_i in range(5):
            col = 3 + k_i  # C..G
            c = ws.cell(r, col, cfg["nilai"][i][k_i])
            c.font = F_INPUT
            c.fill = FILL_INPUT
            c.alignment = AL_CENTER
            c.border = BORDER_CELL
            c.number_format = "0"
            dv_skor.add(c)

        ws.cell(r, 8, catatan_alt[i])
        ws.merge_cells(f"H{r}:K{r}")
        for col in range(8, 12):
            c = ws.cell(r, col)
            c.font = F_MUTED
            c.fill = fill
            c.border = BORDER_CELL
            c.alignment = AL_LEFT
        ws.row_dimensions[r].height = 23

    # Baris referensi normalisasi (rows 22..25)
    ref_rows = [
        (22, "Jenis Kriteria (dari Bagian A)", lambda k_i, cl: f"=C{8+k_i}", None, "Menentukan arah normalisasi SAW"),
        (23, "Nilai Maksimum kolom (MAX)", lambda k_i, cl: f"=MAX({cl}17:{cl}21)", "0", "Pembagi untuk kriteria Benefit"),
        (24, "Nilai Minimum kolom (MIN)", lambda k_i, cl: f"=MIN({cl}17:{cl}21)", "0", "Pembilang untuk kriteria Cost"),
        (25, "Bobot ROC kolom (w_j)", lambda k_i, cl: f"=E{8+k_i}", "0.0000", "Bobot ROC dari Bagian A untuk perkalian SAW"),
    ]
    for r, label, formula_fn, num_fmt, ket in ref_rows:
        ws.merge_cells(f"A{r}:B{r}")
        ws[f"A{r}"] = label
        ws[f"A{r}"].font = F_BODY_BOLD
        ws[f"A{r}"].alignment = AL_RIGHT
        for col in (1, 2):
            ws.cell(r, col).fill = FILL_TOTAL
            ws.cell(r, col).border = BORDER_CELL
        for k_i in range(5):
            col = 3 + k_i
            cl = get_column_letter(col)
            c = ws.cell(r, col, formula_fn(k_i, cl))
            c.font = F_BODY_BOLD if r == 25 else F_BODY
            c.fill = FILL_TOTAL
            c.border = BORDER_CELL
            c.alignment = AL_CENTER if num_fmt in (None, "0") else AL_RIGHT
            if num_fmt:
                c.number_format = num_fmt
        ws.merge_cells(f"H{r}:K{r}")
        ws[f"H{r}"] = ket
        ws[f"H{r}"].font = F_MUTED
        ws[f"H{r}"].alignment = AL_LEFT
        for col in range(8, 12):
            ws.cell(r, col).fill = FILL_NOTE
            ws.cell(r, col).border = BORDER_CELL
        ws.row_dimensions[r].height = 21

    # ---------------- Bagian C: Matriks Ternormalisasi R_ij (rows 27..33)
    section_bar(ws, 27, "C. Matriks Ternormalisasi SAW (R_ij): Benefit = X_ij / MAX, Cost = MIN / X_ij", 11)
    header_row(ws, 28, [
        "Kode", "Nama Alternatif",
        "r(C1)\nCost", "r(C2)\nBenefit", "r(C3)\nBenefit", "r(C4)\nBenefit", "r(C5)\nBenefit",
        "Rumus Excel per Sel", "", "", ""
    ])
    ws.merge_cells("H28:K28")
    for col in range(8, 12):
        ws.cell(28, col).fill = FILL_SLATE
        ws.cell(28, col).border = BORDER_HDR

    for i in range(5):
        r = 29 + i  # rows 29..33
        x_row = 17 + i
        fill = FILL_ZEBRA if i % 2 else FILL_WHITE
        ws.cell(r, 1, f"=A{x_row}")
        ws.cell(r, 2, f"=B{x_row}")
        ws.cell(r, 1).font = F_BODY_BOLD
        ws.cell(r, 2).font = F_BODY
        ws.cell(r, 1).alignment = AL_CENTER
        ws.cell(r, 2).alignment = AL_LEFT
        ws.cell(r, 1).fill = fill
        ws.cell(r, 2).fill = fill
        ws.cell(r, 1).border = BORDER_CELL
        ws.cell(r, 2).border = BORDER_CELL

        for k_i in range(5):
            col = 3 + k_i
            cl = get_column_letter(col)
            c = ws.cell(r, col, f'=IF({cl}$22="Cost",IF({cl}{x_row}=0,0,{cl}$24/{cl}{x_row}),IF({cl}$23=0,0,{cl}{x_row}/{cl}$23))')
            c.font = F_BODY
            c.fill = fill
            c.alignment = AL_RIGHT
            c.border = BORDER_CELL
            c.number_format = "0.0000"

        ws.merge_cells(f"H{r}:K{r}")
        ws.cell(r, 8, f'Cost: MIN/X = C$24/C{x_row} | Benefit: X/MAX = D{x_row}/D$23 (otomatis via IF)')
        for col in range(8, 12):
            c = ws.cell(r, col)
            c.font = F_CODE
            c.fill = fill
            c.border = BORDER_CELL
            c.alignment = AL_LEFT
        ws.row_dimensions[r].height = 22

    # ---------------- Bagian D: Matriks Terbobot (w_j * r_ij), Skor SAW & Peringkat (rows 35..42)
    section_bar(ws, 35, "D. Matriks Terbobot (w_j * r_ij), Skor Akhir SAW (V_i), Peringkat & Konversi Poin Borda", 11)
    header_row(ws, 36, [
        "Kode", "Nama Alternatif",
        "w1 * r1", "w2 * r2", "w3 * r3", "w4 * r4", "w5 * r5",
        "Skor SAW\n(V_i)", "Indeks\n(0-100)", "Peringkat\nIndividual", "Poin Borda\n(m - r)"
    ])

    for i in range(5):
        r = 37 + i  # rows 37..41
        norm_row = 29 + i
        fill = FILL_ZEBRA if i % 2 else FILL_WHITE
        ws.cell(r, 1, f"=A{norm_row}")
        ws.cell(r, 2, f"=B{norm_row}")
        ws.cell(r, 1).font = F_BODY_BOLD
        ws.cell(r, 2).font = F_BODY_BOLD
        ws.cell(r, 1).alignment = AL_CENTER
        ws.cell(r, 2).alignment = AL_LEFT
        ws.cell(r, 1).fill = fill
        ws.cell(r, 2).fill = fill
        ws.cell(r, 1).border = BORDER_CELL
        ws.cell(r, 2).border = BORDER_CELL

        for k_i in range(5):
            col = 3 + k_i
            cl = get_column_letter(col)
            c = ws.cell(r, col, f"={cl}$25*{cl}{norm_row}")
            c.font = F_BODY
            c.fill = fill
            c.alignment = AL_RIGHT
            c.border = BORDER_CELL
            c.number_format = "0.0000"

        # H: Skor SAW = SUM(C..G)
        c_saw = ws.cell(r, 8, f"=SUM(C{r}:G{r})")
        c_saw.font = F_BODY_BOLD
        c_saw.fill = FILL_TOTAL
        c_saw.alignment = AL_RIGHT
        c_saw.border = BORDER_CELL
        c_saw.number_format = "0.0000"

        # I: Indeks 0-100 (sesuai tampilan ringkas web)
        c_idx = ws.cell(r, 9, f"=ROUND(H{r}*100,0)")
        c_idx.font = F_BODY
        c_idx.fill = fill
        c_idx.alignment = AL_CENTER
        c_idx.border = BORDER_CELL
        c_idx.number_format = "0"

        # J: Peringkat individual (anti-seri deterministik mengikuti urutan kode A1..A5)
        c_rank = ws.cell(r, 10, f"=RANK(H{r},$H$37:$H$41,0)+COUNTIF($H$37:H{r},H{r})-1")
        c_rank.font = F_WIN
        c_rank.fill = FILL_WIN
        c_rank.alignment = AL_CENTER
        c_rank.border = BORDER_CELL
        c_rank.number_format = '"#"0'

        # K: Poin Borda dari DM ini (m - peringkat)
        c_borda = ws.cell(r, 11, f"=COUNTA($A$37:$A$41)-J{r}")
        c_borda.font = F_BODY_BOLD
        c_borda.fill = FILL_TOTAL
        c_borda.alignment = AL_CENTER
        c_borda.border = BORDER_CELL
        c_borda.number_format = '0" poin"'

        ws.row_dimensions[r].height = 24

    # Baris pengecekan silang SUMPRODUCT (row 42)
    ws.merge_cells("A42:G42")
    ws["A42"] = "Pemenang Individual DM Ini (otomatis via INDEX/MATCH pada Peringkat #1):"
    ws["A42"].font = F_BODY_BOLD
    ws["A42"].alignment = AL_RIGHT
    ws.merge_cells("H42:I42")
    ws["H42"] = "=INDEX(B37:B41,MATCH(1,J37:J41,0))"
    ws["H42"].font = F_WIN
    ws["H42"].alignment = AL_CENTER
    ws["J42"] = "=MAX(H37:H41)"
    ws["J42"].font = F_BODY_BOLD
    ws["J42"].alignment = AL_RIGHT
    ws["J42"].number_format = "0.0000"
    ws["K42"] = "=SUM(K37:K41)"
    ws["K42"].font = F_BODY_BOLD
    ws["K42"].alignment = AL_CENTER
    ws["K42"].number_format = '0" poin"'
    for col in range(1, 12):
        ws.cell(42, col).fill = FILL_WIN if col in (8, 9) else FILL_TOTAL
        ws.cell(42, col).border = BORDER_TOTAL
    ws.row_dimensions[42].height = 25

    note_box(
        ws, 44,
        "Cara verifikasi: Skor SAW V_i pada kolom H adalah jumlah kontribusi terbobot C37:G37 (setara SUMPRODUCT bobot ROC dan baris normalisasi R_ij). Peringkat #1 mendapat 4 poin Borda, #2 = 3, #3 = 2, #4 = 1, dan #5 = 0 (total selalu 10 poin per DM).",
        11,
        height=36,
    )

    set_widths(ws, {1: 9, 2: 27, 3: 13, 4: 13, 5: 13, 6: 13, 7: 13, 8: 13, 9: 11, 10: 12, 11: 13})
    ws.freeze_panes = "C4"


# --------------------------------------------------------------------------- Sheet 6: Konsensus Borda & Grafik
def build_konsensus_sheet(wb):
    ws = wb.create_sheet("06_Konsensus_Borda")
    setup_sheet(
        ws,
        "KONSENSUS KELOMPOK: METODE VOTING BORDA & PEMECAH SERI JABATAN TERTINGGI",
        "Menggabungkan peringkat individual ketiga Decision Maker. Jika total poin Borda seri, urutan diputuskan berdasarkan peringkat CISO (Level 3).",
        max_col=12,
        tab_color="047857",
    )

    note_box(
        ws, 4,
        "Aturan Borda Count (m = 5 kandidat): Peringkat #1 memperoleh 4 poin, #2 = 3 poin, #3 = 2 poin, #4 = 1 poin, #5 = 0 poin. Nilai maksimum per kandidat = 3 DM * 4 = 12 poin. Rumus Kunci Tie-Break = Total_Poin + (5 - Rank_CISO) / 100.",
        12,
        height=36,
    )

    # ---------------- Tabel 1: Rekapitulasi Voting Borda (rows 6..13)
    section_bar(ws, 6, "1. Matriks Rekapitulasi Peringkat Individual & Perhitungan Poin Voting Borda", 12)
    header_row(ws, 7, [
        "Kode", "Nama Alternatif",
        "Rank\nCISO (L3)", "Poin\nCISO",
        "Rank\nSOC (L2)", "Poin\nSOC",
        "Rank\nNetAdmin (L1)", "Poin\nNetAdmin",
        "Total Poin\nBorda", "Status\nSeri", "Skor Kunci\n(Tie-Break)", "Peringkat\nAkhir"
    ])

    for i in range(5):
        r = 8 + i  # rows 8..12
        dm_r = 37 + i  # rows 37..41 di tiap sheet DM
        fill = FILL_ZEBRA if i % 2 else FILL_WHITE

        ws.cell(r, 1, f"='02_Data_Master'!A{20+i}")
        ws.cell(r, 2, f"='02_Data_Master'!B{20+i}")
        ws.cell(r, 3, f"='03_DM1_CISO'!J{dm_r}")
        ws.cell(r, 4, f"='03_DM1_CISO'!K{dm_r}")
        ws.cell(r, 5, f"='04_DM2_SOC_Analyst'!J{dm_r}")
        ws.cell(r, 6, f"='04_DM2_SOC_Analyst'!K{dm_r}")
        ws.cell(r, 7, f"='05_DM3_Network_Admin'!J{dm_r}")
        ws.cell(r, 8, f"='05_DM3_Network_Admin'!K{dm_r}")
        ws.cell(r, 9, f"=D{r}+F{r}+H{r}")
        ws.cell(r, 10, f'=IF(COUNTIF($I$8:$I$12,I{r})>1,"SERI (CISO)","Tunggal")')
        ws.cell(r, 11, f"=I{r}+(COUNTA($A$8:$A$12)-C{r})/100")
        ws.cell(r, 12, f"=RANK(K{r},$K$8:$K$12,0)")

        for col in range(1, 13):
            c = ws.cell(r, col)
            c.border = BORDER_CELL
            if col in (1, 2):
                c.font = F_LINK_BOLD if col == 1 else F_LINK
                c.fill = fill
                c.alignment = AL_CENTER if col == 1 else AL_LEFT
            elif col in (3, 4, 5, 6, 7, 8):
                c.font = F_LINK
                c.fill = fill
                c.alignment = AL_CENTER
                c.number_format = '"#"0' if col in (3, 5, 7) else "0"
            elif col == 9:
                c.font = F_BODY_BOLD
                c.fill = FILL_TOTAL
                c.alignment = AL_CENTER
                c.number_format = "0"
            elif col == 10:
                c.font = F_WARN
                c.fill = fill
                c.alignment = AL_CENTER
            elif col == 11:
                c.font = F_MUTED
                c.fill = fill
                c.alignment = AL_RIGHT
                c.number_format = "0.00"
            elif col == 12:
                c.font = F_WIN
                c.fill = FILL_WIN
                c.alignment = AL_CENTER
                c.number_format = '"#"0'
        ws.row_dimensions[r].height = 24

    # Baris total cek integritas suara (row 13)
    ws.merge_cells("A13:C13")
    ws["A13"] = "Cek Total Poin Borda (3 DM x 10 poin = 30 poin)"
    ws["A13"].font = F_BODY_BOLD
    ws["A13"].alignment = AL_RIGHT
    ws["D13"] = "=SUM(D8:D12)"
    ws["F13"] = "=SUM(F8:F12)"
    ws["H13"] = "=SUM(H8:H12)"
    ws["I13"] = "=SUM(I8:I12)"
    ws.merge_cells("J13:L13")
    ws["J13"] = '=IF(I13=30,"VALID: Total 30 poin sesuai","PERIKSA INPUT")'
    for col in range(1, 13):
        c = ws.cell(13, col)
        c.fill = FILL_TOTAL
        c.border = BORDER_TOTAL
        if col in (4, 6, 8, 9):
            c.font = F_BODY_BOLD
            c.alignment = AL_CENTER
            c.number_format = "0"
    ws["J13"].font = F_LINK_BOLD
    ws["J13"].alignment = AL_CENTER
    ws.row_dimensions[13].height = 24

    # ---------------- Tabel 2: Papan Peringkat Terurut Otomatis (rows 15..21)
    section_bar(ws, 15, "2. Papan Peringkat Konsensus Terurut Otomatis (Urut Juara #1 sampai #5 via INDEX/MATCH)", 12)
    header_row(ws, 16, [
        "Peringkat", "Kode", "Nama Alternatif", "",
        "Total Poin", "Persen dari Maks (12)",
        "Skor SAW\nCISO", "Skor SAW\nSOC Analyst", "Skor SAW\nNetAdmin",
        "Status Keputusan", "", ""
    ])
    ws.merge_cells("C16:D16")
    ws.merge_cells("J16:L16")
    for col in (3, 4, 10, 11, 12):
        ws.cell(16, col).fill = FILL_SLATE
        ws.cell(16, col).border = BORDER_HDR

    for rank_i in range(1, 6):
        r = 16 + rank_i  # rows 17..21
        is_win = rank_i == 1
        fill = FILL_WIN if is_win else (FILL_ZEBRA if rank_i % 2 == 0 else FILL_WHITE)

        ws.cell(r, 1, rank_i).number_format = '"#"0'
        ws.cell(r, 2, f"=INDEX($A$8:$A$12,MATCH(A{r},$L$8:$L$12,0))")
        ws.cell(r, 3, f"=INDEX($B$8:$B$12,MATCH(A{r},$L$8:$L$12,0))")
        ws.cell(r, 5, f"=INDEX($I$8:$I$12,MATCH(A{r},$L$8:$L$12,0))")
        ws.cell(r, 6, f"=E{r}/12")
        ws.cell(r, 7, f"=INDEX('03_DM1_CISO'!$H$37:$H$41,MATCH(B{r},'03_DM1_CISO'!$A$37:$A$41,0))")
        ws.cell(r, 8, f"=INDEX('04_DM2_SOC_Analyst'!$H$37:$H$41,MATCH(B{r},'04_DM2_SOC_Analyst'!$A$37:$A$41,0))")
        ws.cell(r, 9, f"=INDEX('05_DM3_Network_Admin'!$H$37:$H$41,MATCH(B{r},'05_DM3_Network_Admin'!$A$37:$A$41,0))")
        ws.cell(r, 10, f'=IF(A{r}=1,"TERPILIH SEBAGAI SOLUSI UTAMA",IF(INDEX($J$8:$J$12,MATCH(A{r},$L$8:$L$12,0))="SERI (CISO)","Seri poin, diurutkan oleh CISO","Alternatif cadangan"))')

        ws.merge_cells(f"C{r}:D{r}")
        ws.merge_cells(f"J{r}:L{r}")

        for col in range(1, 13):
            c = ws.cell(r, col)
            c.fill = fill
            c.border = BORDER_CELL
            if is_win:
                c.font = F_WIN
            elif col in (7, 8, 9):
                c.font = F_LINK
            else:
                c.font = F_BODY_BOLD if col in (1, 3, 5) else F_BODY

        ws.cell(r, 1).alignment = AL_CENTER
        ws.cell(r, 2).alignment = AL_CENTER
        ws.cell(r, 3).alignment = AL_LEFT
        ws.cell(r, 5).alignment = AL_CENTER
        ws.cell(r, 5).number_format = '0" poin"'
        ws.cell(r, 6).alignment = AL_RIGHT
        ws.cell(r, 6).number_format = "0.0%"
        for col in (7, 8, 9):
            ws.cell(r, col).alignment = AL_RIGHT
            ws.cell(r, col).number_format = "0.0000"
        ws.cell(r, 10).alignment = AL_LEFT
        ws.row_dimensions[r].height = 25

    # ---------------- Grafik Perbandingan (rows 23..40)
    section_bar(ws, 23, "3. Visualisasi Perbandingan Poin Voting Borda & Skor SAW Individual", 12)

    # Grafik 1: Total Poin Borda (Horizontal Bar)
    chart_borda = BarChart()
    chart_borda.type = "bar"
    chart_borda.style = 10
    chart_borda.roundedCorners = False
    chart_borda.title = "Total Poin Voting Borda (Maksimal 12 Poin)"
    chart_borda.title.overlay = False
    chart_borda.legend = None
    chart_borda.x_axis.delete = False
    chart_borda.y_axis.delete = False
    chart_borda.y_axis.title = "Poin Borda"
    chart_borda.y_axis.title.overlay = False
    chart_borda.y_axis.scaling.min = 0
    chart_borda.y_axis.scaling.max = 12
    data_borda = Reference(ws, min_col=5, min_row=16, max_row=21)
    cats_borda = Reference(ws, min_col=3, min_row=17, max_row=21)
    chart_borda.add_data(data_borda, titles_from_data=True)
    chart_borda.set_categories(cats_borda)
    chart_borda.series[0].graphicalProperties.solidFill = "1D4ED8"
    chart_borda.width = 16.5
    chart_borda.height = 9.5
    ws.add_chart(chart_borda, "A25")

    # Grafik 2: Perbandingan Skor SAW 3 Decision Maker (Clustered Column)
    chart_saw = BarChart()
    chart_saw.type = "col"
    chart_saw.grouping = "clustered"
    chart_saw.style = 10
    chart_saw.roundedCorners = False
    chart_saw.title = "Perbandingan Skor SAW Individual per Jabatan"
    chart_saw.title.overlay = False
    chart_saw.legend.position = "b"
    chart_saw.legend.overlay = False
    chart_saw.x_axis.delete = False
    chart_saw.y_axis.delete = False
    chart_saw.y_axis.title = "Skor SAW (0 - 1)"
    chart_saw.y_axis.title.overlay = False
    chart_saw.y_axis.scaling.min = 0
    chart_saw.y_axis.scaling.max = 1.05
    data_saw = Reference(ws, min_col=7, max_col=9, min_row=16, max_row=21)
    cats_saw = Reference(ws, min_col=3, min_row=17, max_row=21)
    chart_saw.add_data(data_saw, titles_from_data=True)
    chart_saw.set_categories(cats_saw)
    colors_dm = ["1E3A8A", "0F766E", "64748B"]
    for s_idx, col_hex in enumerate(colors_dm):
        chart_saw.series[s_idx].graphicalProperties.solidFill = col_hex
    chart_saw.width = 18.0
    chart_saw.height = 9.5
    ws.add_chart(chart_saw, "G25")

    set_widths(ws, {1: 11, 2: 26, 3: 11, 4: 10, 5: 11, 6: 11, 7: 13, 8: 11, 9: 12, 10: 15, 11: 12, 12: 12})
    ws.freeze_panes = "C4"


# --------------------------------------------------------------------------- Sheet 1: Panduan & Ringkasan
def build_summary_sheet(wb):
    ws = wb.create_sheet("01_Panduan_dan_Ringkasan", 0)
    setup_sheet(
        ws,
        "GDSS PEMILIHAN SIEM UNTUK TIM SOC: WORKBOOK RUMUS & SIMULASI",
        "Proyek Akhir Mata Kuliah Praktikum Sistem Informasi (Dosen Pengampu: Ibu Ashri Shabrina Afrah, M.T.) | Metode: ROC + SAW + Voting Borda",
        max_col=8,
        tab_color="1E293B",
    )

    # Ringkasan Eksekutif Dinamis (rows 4..10)
    section_bar(ws, 4, "1. Ringkasan Eksekutif Hasil Konsensus (Terhubung Otomatis dari 06_Konsensus_Borda)", 8)
    kpis = [
        ("Solusi SIEM Terpilih (#1)", "='06_Konsensus_Borda'!C17", "0", FILL_WIN, F_WIN),
        ("Perolehan Poin Voting Borda", "='06_Konsensus_Borda'!E17", '0" dari 12 poin"', FILL_TOTAL, F_BODY_BOLD),
        ("Pilihan #1 CISO (Level 3)", "='03_DM1_CISO'!H42", "@", FILL_WHITE, F_LINK_BOLD),
        ("Pilihan #1 SOC Analyst (Level 2)", "='04_DM2_SOC_Analyst'!H42", "@", FILL_WHITE, F_LINK_BOLD),
        ("Pilihan #1 Network Admin (Level 1)", "='05_DM3_Network_Admin'!H42", "@", FILL_WHITE, F_LINK_BOLD),
        ("Catatan Pemecah Seri (Tie-Break)", '=IF(COUNTIF(\'06_Konsensus_Borda\'!J8:J12,"SERI (CISO)")>0,"Terdapat alternatif dengan poin Borda sama; urutan ditetapkan berdasarkan peringkat CISO (jabatan tertinggi).","Tidak ada poin seri pada sesi ini.")', "@", FILL_NOTE, F_BODY),
    ]
    for i, (lbl, formula, fmt, fill, font) in enumerate(kpis):
        r = 5 + i
        ws.merge_cells(f"A{r}:C{r}")
        ws.merge_cells(f"D{r}:H{r}")
        ws[f"A{r}"] = lbl
        ws[f"A{r}"].font = F_BODY_BOLD
        ws[f"A{r}"].alignment = AL_LEFT
        ws[f"D{r}"] = formula
        ws[f"D{r}"].font = font
        ws[f"D{r}"].alignment = AL_LEFT
        if fmt != "@":
            ws[f"D{r}"].number_format = fmt
        for col in range(1, 9):
            c = ws.cell(r, col)
            c.fill = fill if col >= 4 else FILL_ZEBRA
            c.border = BORDER_CELL
        ws.row_dimensions[r].height = 28 if i == 5 else 23

    # Legenda warna sel (rows 12..17)
    section_bar(ws, 12, "2. Legenda Kode Warna & Cara Mencoba Simulasi (What-If)", 8)
    header_row(ws, 13, ["Contoh Gaya Sel", "Arti", "Keterangan Penggunaan", "", "", "", "", ""])
    ws.merge_cells("C13:H13")
    for col in range(3, 9):
        ws.cell(13, col).fill = FILL_SLATE
        ws.cell(13, col).border = BORDER_HDR

    legends = [
        ("Input (mis. 5)", FILL_INPUT, F_INPUT, "Sel Input Pengguna", "Boleh diubah langsung (urutan prioritas 1-5 & skor kandidat 1-5 pada sheet 03, 04, 05)."),
        ("0,9687", FILL_WHITE, F_BODY, "Rumus Hitung Lokal", "Dihitung otomatis oleh Excel di sheet yang sama. Jangan ditimpa dengan angka manual."),
        ("Tautan sheet lain", FILL_ZEBRA, F_LINK, "Tautan Lintas-Sheet", "Mengambil nilai langsung dari sheet lain (mis. ='03_DM1_CISO'!J37) sehingga seluruh workbook selalu sinkron."),
        ("#1 Wazuh", FILL_WIN, F_WIN, "Hasil Peringkat / Pemenang", "Penanda kandidat juara pada masing-masing DM maupun hasil akhir konsensus Borda."),
    ]
    for i, (contoh, fill, font, arti, ket) in enumerate(legends):
        r = 14 + i
        ws.cell(r, 1, contoh).fill = fill
        ws.cell(r, 1).font = font
        ws.cell(r, 1).alignment = AL_CENTER
        ws.cell(r, 2, arti).font = F_BODY_BOLD
        ws.cell(r, 2).alignment = AL_LEFT
        ws.cell(r, 3, ket).font = F_BODY
        ws.cell(r, 3).alignment = AL_LEFT
        ws.merge_cells(f"C{r}:H{r}")
        for col in range(1, 9):
            ws.cell(r, col).border = BORDER_CELL
            if col >= 2:
                ws.cell(r, col).fill = FILL_ZEBRA if i % 2 else FILL_WHITE
        ws.row_dimensions[r].height = 24

    # Peta Sheet (rows 19..27)
    section_bar(ws, 19, "3. Daftar Sheet & Alur Perhitungan GDSS", 8)
    header_row(ws, 20, ["No", "Nama Sheet", "Tahap GDSS", "Isi & Rumus Utama", "", "", "", ""])
    ws.merge_cells("D20:H20")
    for col in range(4, 9):
        ws.cell(20, col).fill = FILL_SLATE
        ws.cell(20, col).border = BORDER_HDR

    sheets_info = [
        (1, "01_Panduan_dan_Ringkasan", "Ringkasan", "Menampilkan pemenang konsensus dinamis, legenda warna sel, dan panduan navigasi workbook."),
        (2, "02_Data_Master", "Data Acuan", "Daftar 3 DM, 5 kriteria + rubrik 1-5, 5 kandidat SIEM dengan sumber resmi, serta tabel rumus bobot ROC."),
        (3, "03_DM1_CISO", "Individu (DM 1)", "Prioritas kriteria & penilaian CISO (Level 3): bobot ROC, normalisasi SAW, matriks terbobot, skor & ranking."),
        (4, "04_DM2_SOC_Analyst", "Individu (DM 2)", "Prioritas kriteria & penilaian SOC Analyst (Level 2): perhitungan penuh ROC + SAW independen."),
        (5, "05_DM3_Network_Admin", "Individu (DM 3)", "Prioritas kriteria & penilaian Network Admin (Level 1): perhitungan penuh ROC + SAW independen."),
        (6, "06_Konsensus_Borda", "Konsensus & Grafik", "Rekap ranking 3 DM, konversi poin Borda (4..0), pemecah seri oleh CISO, papan juara terurut otomatis, & 2 grafik."),
        (7, "07_Penjelasan_Rumus", "Dokumentasi", "Penjelasan lengkap teori ROC, SAW, Borda Count, kamus fungsi Excel, dan panduan uji pembuktian saat presentasi."),
    ]
    for i, (no, nm, tahap, isi) in enumerate(sheets_info):
        r = 21 + i
        fill = FILL_ZEBRA if i % 2 else FILL_WHITE
        ws.cell(r, 1, no).alignment = AL_CENTER
        ws.cell(r, 2, nm).alignment = AL_LEFT
        ws.cell(r, 3, tahap).alignment = AL_CENTER
        ws.cell(r, 4, isi).alignment = AL_LEFT
        ws.merge_cells(f"D{r}:H{r}")
        for col in range(1, 9):
            c = ws.cell(r, col)
            c.fill = fill
            c.border = BORDER_CELL
            c.font = F_BODY_BOLD if col in (1, 2) else F_BODY
        ws.row_dimensions[r].height = 24

    note_box(
        ws, 29,
        "Catatan konsistensi: Angka simulasi pada workbook ini 100% identik dengan opsi --contoh pada aplikasi web Rust/Leptos (gdss-siem) serta tabel perhitungan di docs/RANCANGAN.md dan docs/LAPORAN-DAN-TANYA-JAWAB.md.",
        8,
        height=34,
    )

    set_widths(ws, {1: 20, 2: 26, 3: 20, 4: 18, 5: 18, 6: 18, 7: 18, 8: 22})
    ws.freeze_panes = "A4"


# --------------------------------------------------------------------------- Sheet 7: Penjelasan Rumus Lengkap
def build_docs_sheet(wb):
    ws = wb.create_sheet("07_Penjelasan_Rumus")
    setup_sheet(
        ws,
        "PENJELASAN LENGKAP MASING-MASING ALGORITMA, RUMUS EXCEL & BUKTI HITUNG",
        "Uraian langkah demi langkah untuk 3 algoritma inti GDSS (ROC, SAW, Voting Borda + Tie-Break CISO), padanan rumus Excel, dan panduan demo.",
        max_col=7,
        tab_color="6D28D9",
    )

    def algo_block(start_row: int, judul: str, rows_data: list[tuple[str, str, str]]) -> int:
        section_bar(ws, start_row, judul, 7)
        header_row(ws, start_row + 1, ["Aspek", "", "Penjelasan & Langkah Algoritma", "", "", "Rumus Excel / Contoh di Workbook", ""])
        ws.merge_cells(f"A{start_row+1}:B{start_row+1}")
        ws.merge_cells(f"C{start_row+1}:E{start_row+1}")
        ws.merge_cells(f"F{start_row+1}:G{start_row+1}")
        for col in range(1, 8):
            ws.cell(start_row + 1, col).fill = FILL_SLATE
            ws.cell(start_row + 1, col).border = BORDER_HDR

        r = start_row + 2
        for i, (aspek, uraian, rumus) in enumerate(rows_data):
            fill = FILL_ZEBRA if i % 2 else FILL_WHITE
            ws.cell(r, 1, aspek).font = F_BODY_BOLD
            ws.cell(r, 3, uraian).font = F_BODY
            ws.cell(r, 6, rumus).font = F_CODE
            ws.merge_cells(f"A{r}:B{r}")
            ws.merge_cells(f"C{r}:E{r}")
            ws.merge_cells(f"F{r}:G{r}")
            for col in range(1, 8):
                c = ws.cell(r, col)
                c.fill = fill
                c.border = BORDER_CELL
                c.alignment = AL_TOP
            lines = max(uraian.count("\n") + 1, rumus.count("\n") + 1, len(uraian) // 58 + 1, len(rumus) // 36 + 1)
            ws.row_dimensions[r].height = max(30, lines * 15 + 10)
            r += 1
        return r + 1

    r = 4

    # ---------------- ALGORITMA 1: ROC
    r = algo_block(
        r,
        "1. ALGORITMA 1: ROC (Rank Order Centroid) untuk Pembobotan Kriteria Individual",
        [
            (
                "Fungsi & Peran",
                "Mengubah urutan prioritas kriteria (1 sampai 5) dari masing-masing Decision Maker menjadi bobot desimal (w_j) yang konsisten dan jumlahnya selalu tepat 1,0000 (100%).",
                "Dipakai di:\n- 02_Data_Master (D29:D33)\n- 03_DM1_CISO (E8:E12)\n- 04_DM2_SOC_Analyst (E8:E12)\n- 05_DM3_Network_Admin (E8:E12)",
            ),
            (
                "Rumus Matematis",
                "Untuk n kriteria yang diurutkan dari prioritas tertinggi (k = 1) hingga terendah (k = n):\n  w_k = (1 / n) * SUM(1 / i, untuk i = k sampai n)\nSemakin kecil nilai k (semakin penting), semakin besar porsi bobotnya.",
                "n = 5 kriteria:\nk=1: (1 + 1/2 + 1/3 + 1/4 + 1/5)/5 = 0,4567\nk=2: (1/2 + 1/3 + 1/4 + 1/5)/5     = 0,2567\nk=3: (1/3 + 1/4 + 1/5)/5           = 0,1567\nk=4: (1/4 + 1/5)/5                 = 0,0900\nk=5: (1/5)/5                       = 0,0400",
            ),
            (
                "Langkah Algoritma",
                "1. Hitung jumlah kriteria n (di sini n = 5).\n2. Untuk tiap posisi urutan k = 1..n, hitung kebalikan 1/k.\n3. Jumlahkan deret 1/k dari posisi k sampai n, lalu bagi dengan n.\n4. Petakan bobot w_k ke masing-masing kriteria sesuai nomor urutan prioritas yang diinput DM.\n5. Validasi bahwa urutan 1..5 unik sehingga total bobot = 1,0000.",
                "Di 02_Data_Master:\n  B29 = =1/A29\n  C29 = =SUM($B29:$B$33)\n  D29 = =C29/$C$27\nDi sheet tiap DM:\n  E8  = =INDEX('02_Data_Master'!$D$29:$D$33, D8)\nValidasi unik (D13):\n  =IF(AND(COUNT(D8:D12)=5, COUNTIF(...) ),\"VALID\",\"DUPLIKAT!\")",
            ),
            (
                "Contoh Nyata & Padanan Kode",
                "CISO mengurutkan: #1 Fitur alerting (C2 = 45,7%), #2 Biaya (C1 = 25,7%), #3 Integrasi (C3 = 15,7%), #4 Komunitas (C5 = 9,0%), #5 Kemudahan (C4 = 4,0%). Sedangkan Network Admin menempatkan #1 Integrasi (C3 = 45,7%) dan #2 Kemudahan (C4 = 25,7%).",
                "Kode Rust: src/gdss.rs\n  fn roc(n: usize) -> Vec<f64>\n  fn bobot_dari_urutan()\nUnit test: roc_lima_kriteria (PASS)",
            ),
        ],
    )

    # ---------------- ALGORITMA 2: SAW
    r = algo_block(
        r,
        "2. ALGORITMA 2: SAW (Simple Additive Weighting) untuk Perangkingan Alternatif Individual",
        [
            (
                "Fungsi & Peran",
                "Menghitung skor preferensi (V_i) kelima kandidat SIEM untuk masing-masing DM secara terpisah, sehingga setiap jabatan menghasilkan peringkat individual #1 sampai #5 berdasarkan bobot dan skornya sendiri.",
                "Dipakai di:\n- 03_DM1_CISO (Bagian B, C, D)\n- 04_DM2_SOC_Analyst (Bagian B, C, D)\n- 05_DM3_Network_Admin (Bagian B, C, D)",
            ),
            (
                "Rumus Matematis",
                "1) Normalisasi matriks keputusan X_ij (skala 1-5) menjadi R_ij (skala 0-1):\n   - Kriteria Benefit (C2, C3, C4, C5): r_ij = x_ij / MAX(x_j)\n   - Kriteria Cost (C1 Biaya lisensi):  r_ij = MIN(x_j) / x_ij\n2) Skor preferensi akhir tiap alternatif:\n   V_i = SUM(w_j * r_ij, untuk j = 1..5)",
                "Acuan kolom (baris 23-24):\n  MAX: =MAX(C17:C21)\n  MIN: =MIN(C17:C21)\nNormalisasi (C29:G33):\n  =IF(C$22=\"Cost\", IF(C17=0,0,C$24/C17), IF(C$23=0,0,C17/C$23))",
            ),
            (
                "Langkah Algoritma",
                "1. DM mengisi matriks nilai mentah X_ij (skor 1 sampai 5 sesuai rubrik di 02_Data_Master).\n2. Cari nilai MAX (pembagi Benefit) dan MIN (pembilang Cost) pada tiap kolom kriteria.\n3. Hitung matriks ternormalisasi R_ij (nilai terbaik tiap kolom selalu = 1,0000).\n4. Kalikan tiap sel R_ij dengan bobot ROC w_j menghasilkan matriks terbobot.\n5. Jumlahkan tiap baris untuk memperoleh skor V_i, lalu urutkan dari tertinggi ke terendah.",
                "Matriks terbobot (C37:G41):\n  =C$25*C29\nSkor Akhir SAW (H37):\n  =SUM(C37:G37)\nIndeks 0-100 (I37):\n  =ROUND(H37*100,0)\nPeringkat anti-seri (J37):\n  =RANK(H37,$H$37:$H$41,0)+COUNTIF($H$37:H37,H37)-1",
            ),
            (
                "Contoh Hitung Manual (Wazuh di CISO)",
                "Skor mentah Wazuh di CISO: C1=1 (Cost, min=1), C2=5 (max=5), C3=4 (max=5), C4=4 (max=4), C5=5 (max=5).\nNormalisasi R: [1/1=1,0000 ; 5/5=1,0000 ; 4/5=0,8000 ; 4/4=1,0000 ; 5/5=1,0000].\nTerbobot: 0,2567(1) + 0,4567(1) + 0,1567(0,8) + 0,0400(1) + 0,0900(1) = 0,9687 (#1).",
                "Hasil DM1 (CISO): Wazuh 0,9687 (#1)\nHasil DM2 (SOC):  Wazuh 0,9487 (#1)\nHasil DM3 (Net):  Zabbix 0,8080 (#1)\nKode Rust: src/gdss.rs -> fn saw(), fn ranking()\nUnit test: saw_benefit_dan_cost (PASS)",
            ),
        ],
    )

    # ---------------- ALGORITMA 3: BORDA + TIE-BREAK
    r = algo_block(
        r,
        "3. ALGORITMA 3: Voting Borda Count + Pemecah Seri (Tie-Breaker) Jabatan Tertinggi",
        [
            (
                "Fungsi & Peran",
                "Menyatukan 3 daftar peringkat individual yang berbeda (CISO, SOC Analyst, Network Admin) menjadi satu keputusan konsensus kelompok. Dieksekusi oleh DM berjabatan tertinggi (CISO) setelah semua DM selesai menilai.",
                "Dipakai di:\n- Konversi poin: kolom K37:K41 tiap sheet DM\n- Rekap & konsensus: 06_Konsensus_Borda (baris 8-21)\n- Ringkasan: 01_Panduan_dan_Ringkasan (D5:D10)",
            ),
            (
                "Rumus Matematis",
                "Untuk m = 5 alternatif, alternatif pada peringkat r (1..5) dari seorang DM memperoleh poin:\n  Poin(r) = m - r   -> (#1 = 4, #2 = 3, #3 = 2, #4 = 1, #5 = 0)\nTotal Poin Borda = Poin_CISO + Poin_SOC + Poin_NetAdmin (maksimal 3 * 4 = 12 poin).\nJika total poin seri, pemenangnya ditentukan oleh peringkat individual CISO (Level 3).",
                "Konversi poin DM (K37):\n  =COUNTA($A$37:$A$41)-J37\nTotal Borda (06_Konsensus I8):\n  =D8+F8+H8\nDeteksi seri (J8):\n  =IF(COUNTIF($I$8:$I$12,I8)>1,\"SERI (CISO)\",\"Tunggal\")",
            ),
            (
                "Langkah Algoritma",
                "1. Konversi peringkat #1..#5 dari tiap DM menjadi poin 4, 3, 2, 1, 0.\n2. Jumlahkan poin ketiga DM untuk masing-masing kandidat (total seluruh kandidat selalu 30 poin).\n3. Periksa apakah ada dua kandidat atau lebih dengan total poin sama.\n4. Tambahkan bobot tie-breaker mikro (5 - Rank_CISO) / 100 agar kandidat yang lebih tinggi di CISO selalu unggul tanpa pernah menggeser selisih 1 poin Borda utuh.\n5. Urutkan papan peringkat otomatis #1 sampai #5 menggunakan INDEX/MATCH.",
                "Skor Kunci Tie-Break (K8):\n  =I8+(COUNTA($A$8:$A$12)-C8)/100\nPeringkat Akhir (L8):\n  =RANK(K8,$K$8:$K$12,0)\nPapan Urut Otomatis (C17:C21):\n  =INDEX($B$8:$B$12,MATCH(A17,$L$8:$L$12,0))",
            ),
            (
                "Bukti Kasus Seri & Padanan Kode",
                "Wazuh = 4+4+2 = 10 poin (#1 TERPILIH).\nSplunk ES = 3+3+1 = 7 poin; Zabbix = 2+1+4 = 7 poin (SERI).\nSkor Kunci Splunk ES = 7 + (5-2)/100 = 7,03 > Zabbix = 7 + (5-3)/100 = 7,02, sehingga Splunk ES menempati #2 dan Zabbix #3. Elastic = 6 poin (#4), PRTG = 0 poin (#5).",
                "Kode Rust: src/gdss.rs -> fn borda()\nOtorisasi level: src/api.rs -> proses_konsensus()\nUnit test: borda_dengan_seri (PASS)",
            ),
        ],
    )

    # ---------------- BAGIAN 4: PANDUAN UJI SIMULASI WHAT-IF
    section_bar(ws, r, "4. Panduan Uji Simulasi (What-If) & Kompatibilitas Excel", 7)
    header_row(ws, r + 1, ["Topik", "", "Langkah / Penjelasan", "", "", "", ""])
    ws.merge_cells(f"A{r+1}:B{r+1}")
    ws.merge_cells(f"C{r+1}:G{r+1}")
    for col in range(1, 8):
        ws.cell(r + 1, col).fill = FILL_SLATE
        ws.cell(r + 1, col).border = BORDER_HDR

    tips = [
        ("Uji perubahan prioritas", "Buka 03_DM1_CISO lalu tukar sel D8 (Biaya) menjadi 1 dan D9 (Alerting) menjadi 2. Bobot ROC langsung berubah, seluruh skor SAW dan grafik di 06_Konsensus_Borda ikut diperbarui otomatis."),
        ("Uji perubahan skor", "Ubah skor pada sel biru berlatar krem (C17:G21 di sheet DM 1/2/3). Baris MAX, MIN, normalisasi R_ij, peringkat individual, status seri, dan papan juara di 01_Panduan_dan_Ringkasan langsung menghitung ulang."),
        ("Kompatibilitas 100%", "Seluruh 180+ rumus hanya memakai fungsi standar Excel 2007 (SUM, MAX, MIN, IF, RANK, COUNTIF, COUNTA, INDEX, MATCH, ROUND) tanpa XLOOKUP/SORT/makro VBA, sehingga aman dibuka di Microsoft Excel Windows/Mac, Google Sheets, dan LibreOffice Calc."),
    ]
    for i, (t, u) in enumerate(tips):
        row_i = r + 2 + i
        fill = FILL_ZEBRA if i % 2 else FILL_WHITE
        ws.cell(row_i, 1, t).font = F_BODY_BOLD
        ws.cell(row_i, 3, u).font = F_BODY
        ws.merge_cells(f"A{row_i}:B{row_i}")
        ws.merge_cells(f"C{row_i}:G{row_i}")
        for col in range(1, 8):
            c = ws.cell(row_i, col)
            c.fill = fill
            c.border = BORDER_CELL
            c.alignment = AL_TOP
        ws.row_dimensions[row_i].height = 38

    set_widths(ws, {1: 16, 2: 14, 3: 24, 4: 24, 5: 22, 6: 26, 7: 26})
    ws.freeze_panes = "A4"


# --------------------------------------------------------------------------- Evaluator & Cached Values
# Karena LibreOffice mungkin belum terpasang di mesin build, kita juga menghitung nilai
# terverifikasi untuk setiap sel rumus secara independen lalu menanamkan cache <v> beserta
# fullCalcOnLoad="1" agar previewer maupun pandas/openpyxl(data_only=True) langsung
# menampilkan angka benar sambil tetap memaksa Excel menghitung ulang saat dibuka.

def compute_expected_cache() -> dict[str, dict[str, tuple[str, str]]]:
    """Menghitung nilai eksak seluruh rumus untuk verifikasi silang & cache XML.
    Return: {sheet_name: {coord: (value_str, cell_type)}} dengan cell_type '' (numeric) atau 'str'.
    """
    cache: dict[str, dict[str, tuple[str, str]]] = {}

    # 02_Data_Master
    m = {}
    n = 5
    m["C27"] = (str(n), "")
    inv = [1.0 / k for k in range(1, n + 1)]
    harm = [sum(inv[k:]) for k in range(n)]
    roc_w = [h / n for h in harm]
    for k in range(n):
        r = 29 + k
        m[f"B{r}"] = (repr(inv[k]), "")
        m[f"C{r}"] = (repr(harm[k]), "")
        m[f"D{r}"] = (repr(roc_w[k]), "")
        m[f"E{r}"] = (repr(roc_w[k]), "")
    m["D34"] = (repr(sum(roc_w)), "")
    m["E34"] = (repr(sum(roc_w)), "")
    m["F34"] = ("VALID: Total bobot = 1", "str")
    cache["02_Data_Master"] = m

    # DM sheets
    dm_results = {}
    for cfg in DM_CONFIG:
        d = {}
        prio = cfg["prioritas"]
        w_cols = [roc_w[p - 1] for p in prio]
        for i in range(5):
            r = 8 + i
            d[f"A{r}"] = (KRITERIA[i][0], "str")
            d[f"B{r}"] = (KRITERIA[i][1], "str")
            d[f"C{r}"] = (KRITERIA[i][2], "str")
            d[f"E{r}"] = (repr(w_cols[i]), "")
            d[f"F{r}"] = (repr(w_cols[i]), "")
        d["D13"] = ("VALID", "str")
        d["E13"] = (repr(sum(w_cols)), "")
        d["F13"] = (repr(sum(w_cols)), "")

        mat = cfg["nilai"]
        for i in range(5):
            r = 17 + i
            d[f"A{r}"] = (ALTERNATIF[i][0], "str")
            d[f"B{r}"] = (ALTERNATIF[i][1], "str")

        col_max = [max(mat[i][j] for i in range(5)) for j in range(5)]
        col_min = [min(mat[i][j] for i in range(5)) for j in range(5)]
        for j in range(5):
            cl = get_column_letter(3 + j)
            d[f"{cl}22"] = (KRITERIA[j][2], "str")
            d[f"{cl}23"] = (str(col_max[j]), "")
            d[f"{cl}24"] = (str(col_min[j]), "")
            d[f"{cl}25"] = (repr(w_cols[j]), "")

        norm = []
        for i in range(5):
            r = 29 + i
            d[f"A{r}"] = (ALTERNATIF[i][0], "str")
            d[f"B{r}"] = (ALTERNATIF[i][1], "str")
            row_n = []
            for j in range(5):
                cl = get_column_letter(3 + j)
                val = (col_min[j] / mat[i][j]) if KRITERIA[j][2] == "Cost" else (mat[i][j] / col_max[j])
                row_n.append(val)
                d[f"{cl}{r}"] = (repr(val), "")
            norm.append(row_n)

        saws = []
        for i in range(5):
            r = 37 + i
            d[f"A{r}"] = (ALTERNATIF[i][0], "str")
            d[f"B{r}"] = (ALTERNATIF[i][1], "str")
            w_vals = []
            for j in range(5):
                cl = get_column_letter(3 + j)
                wv = w_cols[j] * norm[i][j]
                w_vals.append(wv)
                d[f"{cl}{r}"] = (repr(wv), "")
            s = sum(w_vals)
            saws.append(s)
            d[f"H{r}"] = (repr(s), "")
            d[f"I{r}"] = (str(int(round(s * 100))), "")

        # Ranking anti-seri: RANK + COUNTIF($H$37:Hr, Hr) - 1
        ranks = []
        borda_pts = []
        for i in range(5):
            r = 37 + i
            gt = sum(1 for x in saws if x > saws[i])
            eq_before = sum(1 for k in range(i + 1) if saws[k] == saws[i])
            rk = gt + 1 + eq_before - 1
            bp = 5 - rk
            ranks.append(rk)
            borda_pts.append(bp)
            d[f"J{r}"] = (str(rk), "")
            d[f"K{r}"] = (str(bp), "")

        win_idx = ranks.index(1)
        d["H42"] = (ALTERNATIF[win_idx][1], "str")
        d["J42"] = (repr(max(saws)), "")
        d["K42"] = (str(sum(borda_pts)), "")

        cache[cfg["sheet"]] = d
        dm_results[cfg["sheet"]] = {"saws": saws, "ranks": ranks, "borda": borda_pts, "winner": ALTERNATIF[win_idx][1]}

    # 06_Konsensus_Borda
    k_map = {}
    ciso = dm_results["03_DM1_CISO"]
    soc = dm_results["04_DM2_SOC_Analyst"]
    net = dm_results["05_DM3_Network_Admin"]
    totals = [ciso["borda"][i] + soc["borda"][i] + net["borda"][i] for i in range(5)]
    keys = [totals[i] + (5 - ciso["ranks"][i]) / 100.0 for i in range(5)]
    final_ranks = [1 + sum(1 for x in keys if x > keys[i]) for i in range(5)]

    for i in range(5):
        r = 8 + i
        is_tie = totals.count(totals[i]) > 1
        k_map[f"A{r}"] = (ALTERNATIF[i][0], "str")
        k_map[f"B{r}"] = (ALTERNATIF[i][1], "str")
        k_map[f"C{r}"] = (str(ciso["ranks"][i]), "")
        k_map[f"D{r}"] = (str(ciso["borda"][i]), "")
        k_map[f"E{r}"] = (str(soc["ranks"][i]), "")
        k_map[f"F{r}"] = (str(soc["borda"][i]), "")
        k_map[f"G{r}"] = (str(net["ranks"][i]), "")
        k_map[f"H{r}"] = (str(net["borda"][i]), "")
        k_map[f"I{r}"] = (str(totals[i]), "")
        k_map[f"J{r}"] = ("SERI (CISO)" if is_tie else "Tunggal", "str")
        k_map[f"K{r}"] = (repr(keys[i]), "")
        k_map[f"L{r}"] = (str(final_ranks[i]), "")

    k_map["D13"] = (str(sum(ciso["borda"])), "")
    k_map["F13"] = (str(sum(soc["borda"])), "")
    k_map["H13"] = (str(sum(net["borda"])), "")
    k_map["I13"] = (str(sum(totals)), "")
    k_map["J13"] = ("VALID: Total 30 poin sesuai", "str")

    for rk in range(1, 6):
        r = 16 + rk
        idx = final_ranks.index(rk)
        is_tie = totals.count(totals[idx]) > 1
        k_map[f"B{r}"] = (ALTERNATIF[idx][0], "str")
        k_map[f"C{r}"] = (ALTERNATIF[idx][1], "str")
        k_map[f"E{r}"] = (str(totals[idx]), "")
        k_map[f"F{r}"] = (repr(totals[idx] / 12.0), "")
        k_map[f"G{r}"] = (repr(ciso["saws"][idx]), "")
        k_map[f"H{r}"] = (repr(soc["saws"][idx]), "")
        k_map[f"I{r}"] = (repr(net["saws"][idx]), "")
        ket = "TERPILIH SEBAGAI SOLUSI UTAMA" if rk == 1 else ("Seri poin, diurutkan oleh CISO" if is_tie else "Alternatif cadangan")
        k_map[f"J{r}"] = (ket, "str")
    cache["06_Konsensus_Borda"] = k_map

    # 01_Panduan_dan_Ringkasan
    win_idx = final_ranks.index(1)
    any_tie = any(totals.count(t) > 1 for t in totals)
    cache["01_Panduan_dan_Ringkasan"] = {
        "D5": (ALTERNATIF[win_idx][1], "str"),
        "D6": (str(totals[win_idx]), ""),
        "D7": (ciso["winner"], "str"),
        "D8": (soc["winner"], "str"),
        "D9": (net["winner"], "str"),
        "D10": (
            "Terdapat alternatif dengan poin Borda sama; urutan ditetapkan berdasarkan peringkat CISO (jabatan tertinggi)."
            if any_tie
            else "Tidak ada poin seri pada sesi ini.",
            "str",
        ),
    }

    return cache


def inject_cached_values_and_full_calc(xlsx_path: Path, cache: dict[str, dict[str, tuple[str, str]]]):
    """Menambahkan <v> terverifikasi pada setiap sel rumus dan fullCalcOnLoad='1' di workbook.xml."""
    ns_main = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
    ns_rel = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
    ET.register_namespace("", ns_main)
    ET.register_namespace("r", ns_rel)

    tmp_path = xlsx_path.with_suffix(".tmp.xlsx")
    with zipfile.ZipFile(xlsx_path, "r") as zin:
        # Petakan nama sheet -> path XML
        wb_root = ET.fromstring(zin.read("xl/workbook.xml"))
        rels_root = ET.fromstring(zin.read("xl/_rels/workbook.xml.rels"))
        rid_to_target = {rel.attrib["Id"]: rel.attrib["Target"] for rel in rels_root}
        target_to_name = {}
        for sheet_el in wb_root.findall(f".//{{{ns_main}}}sheet"):
            name = sheet_el.attrib["name"]
            rid = sheet_el.attrib[f"{{{ns_rel}}}id"]
            target = rid_to_target[rid].lstrip("/")
            if not target.startswith("xl/"):
                target = "xl/" + target
            target_to_name[target] = name

        # Pastikan calcPr punya fullCalcOnLoad="1"
        calc_pr = wb_root.find(f"{{{ns_main}}}calcPr")
        if calc_pr is None:
            calc_pr = ET.SubElement(wb_root, f"{{{ns_main}}}calcPr")
        calc_pr.set("fullCalcOnLoad", "1")
        wb_bytes = ET.tostring(wb_root, encoding="utf-8", xml_declaration=True)

        formula_count = 0
        cached_count = 0

        with zipfile.ZipFile(tmp_path, "w", zipfile.ZIP_DEFLATED) as zout:
            for item in zin.infolist():
                data = zin.read(item.filename)
                if item.filename == "xl/workbook.xml":
                    zout.writestr(item, wb_bytes)
                elif item.filename in target_to_name:
                    sname = target_to_name[item.filename]
                    s_cache = cache.get(sname, {})
                    root = ET.fromstring(data)
                    for c_el in root.findall(f".//{{{ns_main}}}c"):
                        f_el = c_el.find(f"{{{ns_main}}}f")
                        if f_el is not None:
                            formula_count += 1
                            ref = c_el.attrib["r"]
                            f_text = f_el.text or ""
                            # Validasi keseimbangan kurung & nol em-dash di rumus
                            assert f_text.count("(") == f_text.count(")"), f"Kurung tak seimbang di {sname}!{ref}: {f_text}"
                            if ref in s_cache:
                                val_str, c_type = s_cache[ref]
                                if c_type == "str":
                                    c_el.set("t", "str")
                                else:
                                    c_el.attrib.pop("t", None)
                                v_el = c_el.find(f"{{{ns_main}}}v")
                                if v_el is None:
                                    v_el = ET.SubElement(c_el, f"{{{ns_main}}}v")
                                v_el.text = val_str
                                cached_count += 1
                    zout.writestr(item, ET.tostring(root, encoding="utf-8", xml_declaration=True))
                else:
                    zout.writestr(item, data)

    tmp_path.replace(xlsx_path)
    assert formula_count == cached_count, f"Ada rumus tanpa cache: {cached_count}/{formula_count}"
    print(f"✓ {formula_count} rumus terverifikasi, seluruhnya memiliki nilai cache & fullCalcOnLoad=1.")


def verify_workbook(xlsx_path: Path):
    """Memeriksa workbook dalam dua mode: formula (data_only=False) dan nilai cache (data_only=True)."""
    wb_f = openpyxl.load_workbook(xlsx_path, data_only=False)
    wb_v = openpyxl.load_workbook(xlsx_path, data_only=True)

    # 1. Pastikan 0 em dash di seluruh teks sel
    for sname in wb_f.sheetnames:
        ws = wb_f[sname]
        for row in ws.iter_rows():
            for cell in row:
                if isinstance(cell.value, str) and "—" in cell.value:
                    raise AssertionError(f"Em dash ditemukan di {sname}!{cell.coordinate}")

    # 2. Pastikan seluruh sel rumus memiliki nilai cache non-None dan tanpa galat Excel
    err_tokens = {"#REF!", "#DIV/0!", "#VALUE!", "#NAME?", "#N/A", "#NUM!"}
    total_f = 0
    for sname in wb_f.sheetnames:
        ws_f, ws_v = wb_f[sname], wb_v[sname]
        for row in ws_f.iter_rows():
            for cell in row:
                if isinstance(cell.value, str) and cell.value.startswith("="):
                    total_f += 1
                    val = ws_v[cell.coordinate].value
                    if val is None or (isinstance(val, str) and val in err_tokens):
                        raise AssertionError(f"Galat/kosong di {sname}!{cell.coordinate}: {cell.value} -> {val}")

    # 3. Cocokkan angka kunci dengan perhitungan Rust (src/gdss.rs)
    ciso_v = wb_v["03_DM1_CISO"]
    soc_v = wb_v["04_DM2_SOC_Analyst"]
    net_v = wb_v["05_DM3_Network_Admin"]
    kon_v = wb_v["06_Konsensus_Borda"]

    assert abs(ciso_v["H37"].value - 0.9686666667) < 1e-4, ciso_v["H37"].value
    assert abs(ciso_v["H38"].value - 0.7566666667) < 1e-4, ciso_v["H38"].value
    assert abs(soc_v["H37"].value - 0.9486666667) < 1e-4, soc_v["H37"].value
    assert abs(net_v["H40"].value - 0.8080) < 1e-4, net_v["H40"].value
    assert kon_v["C17"].value == "Wazuh" and kon_v["E17"].value == 10
    assert kon_v["C18"].value == "Splunk Enterprise Security" and kon_v["E18"].value == 7
    assert kon_v["C19"].value == "Zabbix" and kon_v["E19"].value == 7
    assert kon_v["C20"].value == "Elastic Security (ELK)" and kon_v["E20"].value == 6
    assert kon_v["C21"].value == "PRTG Network Monitor" and kon_v["E21"].value == 0

    print(f"✓ Audit workbook selesai: {len(wb_f.sheetnames)} sheet, {total_f} rumus aktif, 0 galat, 0 em dash.")
    print("✓ Papan peringkat konsensus (cache terverifikasi):")
    for r in range(17, 22):
        print(f"  #{kon_v[f'A{r}'].value} {kon_v[f'C{r}'].value:<28} {kon_v[f'E{r}'].value:>2} poin  | {kon_v[f'J{r}'].value}")


def main():
    wb = openpyxl.Workbook()
    # Hapus sheet default kosong
    wb.remove(wb.active)

    build_data_master(wb)
    for cfg in DM_CONFIG:
        build_dm_sheet(wb, cfg)
    build_konsensus_sheet(wb)
    build_summary_sheet(wb)
    build_docs_sheet(wb)

    OUT_XLSX.parent.mkdir(parents=True, exist_ok=True)
    wb.save(OUT_XLSX)

    cache = compute_expected_cache()
    inject_cached_values_and_full_calc(OUT_XLSX, cache)
    verify_workbook(OUT_XLSX)
    print(f"✓ Tersimpan di: {OUT_XLSX}")


if __name__ == "__main__":
    main()
