#!/usr/bin/env python3
"""Open Frame Studio — Python sidecar for IFC, DXF, and PDF generation."""

import argparse
import json
import sys


def main():
    parser = argparse.ArgumentParser(description="Open Frame Studio Python Sidecar")
    subparsers = parser.add_subparsers(dest="command")

    # generate-ifc
    ifc_parser = subparsers.add_parser("generate-ifc")
    ifc_parser.add_argument("--output", required=True)
    ifc_parser.add_argument("--kozijn-json", required=True)

    # generate-dxf
    dxf_parser = subparsers.add_parser("generate-dxf")
    dxf_parser.add_argument("--output", required=True)
    dxf_parser.add_argument("--kozijn-json", required=True)

    # generate-kozijnstaat
    staat_parser = subparsers.add_parser("generate-kozijnstaat")
    staat_parser.add_argument("--output", required=True)
    staat_parser.add_argument("--format", choices=["pdf", "xlsx"], default="pdf")
    staat_parser.add_argument("--project-json", required=True)

    # generate-werkplaats-tekening
    werk_parser = subparsers.add_parser("generate-werkplaats-tekening")
    werk_parser.add_argument("--output", required=True)
    werk_parser.add_argument("--kozijn-json", required=True)
    werk_parser.add_argument("--project-json", required=True)

    # generate-gltf
    gltf_parser = subparsers.add_parser("generate-gltf")
    gltf_parser.add_argument("--output", required=True)
    gltf_parser.add_argument("--kozijn-json", required=True)

    # generate-production-lists
    prod_parser = subparsers.add_parser("generate-production-lists")
    prod_parser.add_argument("--output", required=True)
    prod_parser.add_argument("--format", choices=["pdf", "xlsx", "csv"], default="pdf")
    prod_parser.add_argument("--production-json", required=True)

    args = parser.parse_args()

    if args.command == "generate-ifc":
        kozijn = json.loads(args.kozijn_json)
        from ofs_ifc.generator import generate_ifc
        generate_ifc(kozijn, args.output)
        print(json.dumps({"status": "ok", "path": args.output}))

    elif args.command == "generate-dxf":
        kozijn = json.loads(args.kozijn_json)
        from ofs_dxf.generator import generate_dxf
        generate_dxf(kozijn, args.output)
        print(json.dumps({"status": "ok", "path": args.output}))

    elif args.command == "generate-kozijnstaat":
        project = json.loads(args.project_json)
        from ofs_pdf.kozijnstaat import generate_kozijnstaat
        generate_kozijnstaat(project, args.output, args.format)
        print(json.dumps({"status": "ok", "path": args.output}))

    elif args.command == "generate-werkplaats-tekening":
        kozijn = json.loads(args.kozijn_json)
        project = json.loads(args.project_json)
        from ofs_pdf.werkplaats_tekening import generate_workshop_drawing
        generate_workshop_drawing(kozijn, project, args.output)
        print(json.dumps({"status": "ok", "path": args.output}))

    elif args.command == "generate-gltf":
        kozijn = json.loads(args.kozijn_json)
        from ofs_gltf.generator import generate_gltf
        generate_gltf(kozijn, args.output)
        print(json.dumps({"status": "ok", "path": args.output}))

    elif args.command == "generate-production-lists":
        prod_data = json.loads(args.production_json)
        # Ensure it's a list
        if isinstance(prod_data, dict):
            prod_data = [prod_data]

        if args.format == "pdf":
            from ofs_production.pdf_generator import generate_production_pdf
            generate_production_pdf(prod_data, args.output)
        elif args.format == "xlsx":
            from ofs_production.xlsx_generator import generate_production_xlsx
            generate_production_xlsx(prod_data, args.output)
        elif args.format == "csv":
            from ofs_production.csv_generator import generate_production_csv
            generate_production_csv(prod_data, args.output)
        print(json.dumps({"status": "ok", "path": args.output}))

    else:
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
