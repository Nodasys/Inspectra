"""
Example: Memory scanner with Python
"""

import inspectra

def main():
    inspectra.init()
    
    print("Inspectra Memory Scanner Example\n")
    
    # Get target process
    pid = int(input("Enter process PID: "))
    
    try:
        scanner = inspectra.Scanner(pid)
        print(f"Attached to process: {pid}\n")
        
        while True:
            value_input = input("Enter value to scan (or 'quit'): ")
            
            if value_input.lower() == 'quit':
                break
            
            try:
                value = int(value_input)
                results = scanner.scan_i32(value)
                
                print(f"Found {len(results)} results")
                
                # Display first 10
                for i, addr in enumerate(results[:10]):
                    print(f"  [{i}] Address: 0x{addr:X}")
                
                if len(results) > 10:
                    print(f"  ... and {len(results) - 10} more")
                
            except ValueError:
                print("Invalid value")
    
    except Exception as e:
        print(f"Error: {e}")
    
    print("Exiting...")

if __name__ == "__main__":
    main()
