import os
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.colors import hsv_to_rgb

# Constants for easy editing
GRID_SIZE = 12  # Number of squares along each side of the grid
X_RANGE = (0, 11)  # Range of x-values to match the grid indices
Y_RANGE = (0, 11)  # Range of y-values to match the grid indices
SIGMA_X_RANGE = (3, 5)  # Range of spread in the x-direction
SIGMA_Y_RANGE = (3, 5)  # Range of spread in the y-direction
AMPLITUDE_RANGE = (0.5, 2)  # Range of the amplitude (height) of the hump
SEED_BYTES = 4  # Number of bytes for generating a truly random seed
# Portion of randomness to add to spread and hump height (0.0 to 1.0)
RANDOMNESS_FACTOR = 0.1
ENABLE_RANDOMNESS = True  # Enable or disable randomness


def generate_random_seed(seed_bytes):
    return int.from_bytes(os.urandom(seed_bytes), 'little')


def initialize_grid(x_range, y_range, grid_size):
    x = np.linspace(x_range[0], x_range[1], grid_size)
    y = np.linspace(y_range[0], y_range[1], grid_size)
    return np.meshgrid(x, y)


def gaussian_2d(x, y, x0, y0, sigma_x, sigma_y, A):
    return A * np.exp(-((x - x0)**2 / (2 * sigma_x**2) + (y - y0)**2 / (2 * sigma_y**2)))


def add_randomness(value, randomness_factor):
    return value * (1 + np.random.uniform(-randomness_factor, randomness_factor))


def generate_gaussian_parameters(x_range, y_range, sigma_x_range, sigma_y_range, amplitude_range, randomness_factor, enable_randomness):
    x0, y0 = np.random.uniform(x_range[0], x_range[1], 2)
    sigma_x, sigma_y = np.random.uniform(sigma_x_range[0], sigma_x_range[1], 2)
    A = np.random.uniform(amplitude_range[0], amplitude_range[1])

    if enable_randomness:
        sigma_x = add_randomness(sigma_x, randomness_factor)
        sigma_y = add_randomness(sigma_y, randomness_factor)
        A = add_randomness(A, randomness_factor)

    return x0, y0, sigma_x, sigma_y, A


def normalize_data(data):
    return (data - data.min()) / (data.max() - data.min())


def convert_to_rgb(hsv_image):
    return (hsv_to_rgb(hsv_image) * 255).astype(int)


def find_winner_coordinate(z):
    winner_index = np.unravel_index(np.argmax(z, axis=None), z.shape)
    return winner_index[1], winner_index[0]


def print_rgb_grid(rgb_image_255, grid_size):
    print("RGB values for each square in the 12x12 grid (0,0 at bottom-left):")
    for i in range(grid_size-1, -1, -1):
        for j in range(grid_size):
            print(f"{rgb_image_255[i, j]}", end=' ')
        print()


def plot_results(x, y, z, rgb_image, x_range, y_range):
    fig = plt.figure(figsize=(14, 7))

    # 3D Surface plot
    ax1 = fig.add_subplot(131, projection='3d')
    ax1.plot_surface(x, y, z, cmap='viridis')
    ax1.set_title('3D Surface Plot')
    ax1.set_xlabel('x')
    ax1.set_ylabel('y')
    ax1.set_zlabel('z')

    # 2D Height Map
    ax2 = fig.add_subplot(132)
    cax2 = ax2.imshow(
        z, extent=[x_range[0], x_range[1], y_range[0], y_range[1]], origin='lower')
    fig.colorbar(cax2, ax=ax2)
    ax2.set_title('Height Map')
    ax2.set_xlabel('x')
    ax2.set_ylabel('y')

    # Color Heatmap
    ax3 = fig.add_subplot(133)
    ax3.imshow(rgb_image, extent=[
               x_range[0], x_range[1], y_range[0], y_range[1]], origin='lower')
    ax3.set_title('Heatmap')
    ax3.set_xlabel('x')
    ax3.set_ylabel('y')

    plt.show()


def main():
    random_seed = generate_random_seed(SEED_BYTES)
    np.random.seed(random_seed)

    x, y = initialize_grid(X_RANGE, Y_RANGE, GRID_SIZE)

    x0, y0, sigma_x, sigma_y, A = generate_gaussian_parameters(
        X_RANGE, Y_RANGE, SIGMA_X_RANGE, SIGMA_Y_RANGE, AMPLITUDE_RANGE, RANDOMNESS_FACTOR, ENABLE_RANDOMNESS
    )

    z = gaussian_2d(x, y, x0, y0, sigma_x, sigma_y, A)
    z_normalized = normalize_data(z)

    hue = 0.66 * (1 - z_normalized)  # 0.66 is blue, 0 is red
    saturation = np.ones_like(z)
    value = np.ones_like(z)
    hsv_image = np.stack((hue, saturation, value), axis=-1)
    rgb_image_255 = convert_to_rgb(hsv_image)

    winner_coordinate = find_winner_coordinate(z)

    print_rgb_grid(rgb_image_255, GRID_SIZE)

    print(f"Winner coordinate (grid index): {winner_coordinate}")
    print(f"Continuous max point (x0, y0): ({x0}, {y0})")

    plot_results(x, y, z, rgb_image_255, X_RANGE, Y_RANGE)

    print(f"Random seed: {random_seed}")
    print(f"x0: {x0}, y0: {y0}")
    print(f"sigma_x: {sigma_x}, sigma_y: {sigma_y}")
    print(f"A: {A}")
    print(f"Winner coordinate (grid index): {winner_coordinate}")
    print(f"Continuous max point (x0, y0): ({x0}, {y0})")


if __name__ == "__main__":
    main()
