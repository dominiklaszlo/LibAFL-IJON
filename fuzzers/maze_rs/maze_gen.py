#!/usr/bin/python3

"""
    @author: Dominik László + Gemini 3.1 Pro
"""

import random
import sys

def setup_recursion(size):
    limit = (size * size) + 10000
    sys.setrecursionlimit(limit)

def generate_maze(dim):
    maze = [[1 for _ in range(dim)] for _ in range(dim)]
    visited = set()

    def carve_passages(cx, cy):
        visited.add((cx, cy))
        maze[cy][cx] = 0
        
        directions = [(0, -2), (0, 2), (-2, 0), (2, 0)]
        random.shuffle(directions)

        for dx, dy in directions:
            nx, ny = cx + dx, cy + dy
            if 1 <= nx < dim - 1 and 1 <= ny < dim - 1:
                if (nx, ny) not in visited:
                    maze[cy + dy // 2][cx + dx // 2] = 0
                    carve_passages(nx, ny)

    carve_passages(1, 1)

    exit_coords = (None, None)
    for x in range(1, dim - 1):
        if maze[1][x] == 0:
            maze[0][x] = 3
            exit_coords = (0, x) # row, col
            break

    start_coords = (None, None)
    placed_start = False
    for y in range(dim - 2, 0, -1):
        for x in range(dim - 2, 0, -1):
            if maze[y][x] == 0:
                maze[y][x] = 2
                start_coords = (y, x) # row, col
                placed_start = True
                break
        if placed_start:
            break

    return maze, start_coords, exit_coords

def save_maze(maze, size):
    filename = f"maze_{size}x{size}.txt"
    with open(filename, 'w') as f:
        f.write("[\n")
        for row in maze:
            formatted_row = "    [" + ", ".join(map(str, row)) + "],"
            f.write(formatted_row + "\n")
        f.write("]")
    return filename

def main():
    print("--- Maze Generator Tool ---")
    try:
        user_input = input("Enter maze size (5 - 100): ").strip()
        size = int(user_input)
        
        if size < 5:
            print("Size too small.")
            return

        setup_recursion(size)
        
        print(f"\nGenerating {size}x{size} maze...")
        maze_data, start_pos, exit_pos = generate_maze(size)
        
        file_path = save_maze(maze_data, size)

        # Output the results
        print("-" * 45)
        print(f"SUCCESS!")
        print(f"File Saved:     {file_path}")
        print(f"Start Pos (2):  Row {start_pos[0]}, Col {start_pos[1]}")
        print(f"Exit Pos  (3):  Row {exit_pos[0]}, Col {exit_pos[1]}")
        print("-" * 45)

    except ValueError:
        print("Invalid input. WHOLE NUMBER ONLY!")
    except Exception as e:
        print(f"An error occurred: {e}")

if __name__ == "__main__":
    main()
