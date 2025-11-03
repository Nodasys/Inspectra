"""
Example: Process listing with Python bindings
"""

import inspectra

def main():
    # Initialize Inspectra
    inspectra.init()
    print(f"Inspectra version: {inspectra.version()}\n")

    # Create process manager
    manager = inspectra.ProcessManager()

    # List all processes
    print("Listing processes...\n")
    processes = manager.list_processes()

    # Sort by name
    processes.sort(key=lambda p: p.name.lower())

    # Display
    print(f"{'PID':<10} {'Name':<30} {'Path'}")
    print("=" * 80)

    for proc in processes[:20]:
        print(f"{proc.pid:<10} {proc.name:<30} {proc.path}")

    print(f"\nTotal: {len(processes)} processes")

    # Find specific process
    print("\n" + "=" * 80)
    search_name = input("\nSearch for process (name): ")
    
    if search_name:
        results = manager.find_by_name(search_name)
        print(f"\nFound {len(results)} process(es):")
        
        for proc in results:
            print(f"  PID: {proc.pid}, Name: {proc.name}")

if __name__ == "__main__":
    main()
