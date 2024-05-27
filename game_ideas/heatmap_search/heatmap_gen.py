import os
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.colors import hsv_to_rgb

# Constants for easy editing
GRID_SIZE = 12  # Number of squares along each side of the grid
X_RANGE = (0, 11)  # Range of x-values to match the grid indices
Y_RANGE = (0, 11)  # Range of y-values to match the grid indices
# Range of spread in the x-direction. Increase to make the hump wider in the x-direction.
SIGMA_X_RANGE = (3, 5)
# Range of spread in the y-direction. Increase to make the hump wider in the y-direction.
SIGMA_Y_RANGE = (3, 5)
# Range of the amplitude (height) of the hump. Increase to make the hump taller.
AMPLITUDE_RANGE = (0.5, 2)
SEED_BYTES = 4  # Number of bytes for generating a truly random seed

# Generate a truly random seed
random_seed = int.from_bytes(os.urandom(SEED_BYTES), 'little')

# Set the random seed for numpy
np.random.seed(random_seed)

# Define the grid
x = np.linspace(X_RANGE[0], X_RANGE[1], GRID_SIZE)
y = np.linspace(Y_RANGE[0], Y_RANGE[1], GRID_SIZE)
x, y = np.meshgrid(x, y)

# Define the Gaussian function


def gaussian_2d(x, y, x0, y0, sigma_x, sigma_y, A):
    return A * np.exp(-((x - x0)**2 / (2 * sigma_x**2) + (y - y0)**2 / (2 * sigma_y**2)))


# Generate random parameters for the Gaussian
# Random center of the hump within the grid range
x0, y0 = np.random.uniform(X_RANGE[0], X_RANGE[1], 2)
# Random spread parameters within the specified range
sigma_x, sigma_y = np.random.uniform(SIGMA_X_RANGE[0], SIGMA_X_RANGE[1], 2)
# Random amplitude within the specified range
A = np.random.uniform(AMPLITUDE_RANGE[0], AMPLITUDE_RANGE[1])

# Calculate the z values
z = gaussian_2d(x, y, x0, y0, sigma_x, sigma_y, A)

# Normalize z to range [0, 1]
z_normalized = (z - z.min()) / (z.max() - z.min())

# Convert z to HSV color
hue = 0.66 * (1 - z_normalized)  # 0.66 is blue, 0 is red
saturation = np.ones_like(z)
value = np.ones_like(z)
hsv_image = np.stack((hue, saturation, value), axis=-1)
rgb_image = hsv_to_rgb(hsv_image)

# Convert rgb_image to RGB values in the range [0, 255]
rgb_image_255 = (rgb_image * 255).astype(int)

# Find the winner coordinate (index of the highest value in z)
winner_index = np.unravel_index(np.argmax(z, axis=None), z.shape)
# Swap to match plot coordinates
winner_coordinate = (winner_index[1], winner_index[0])

# Print the RGB values formatted in a grid
print("RGB values for each square in the 12x12 grid (0,0 at bottom-left):")
for i in range(GRID_SIZE-1, -1, -1):
    for j in range(GRID_SIZE):
        print(f"{rgb_image_255[i, j]}", end=' ')
    print()

# Print the winner coordinate (discrete grid) and continuous max point
print(f"Winner coordinate (grid index): {winner_coordinate}")
print(f"Continuous max point (x0, y0): ({x0}, {y0})")

# Plot the results
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
cax2 = ax2.imshow(z, extent=[X_RANGE[0], X_RANGE[1],
                  Y_RANGE[0], Y_RANGE[1]], origin='lower')
fig.colorbar(cax2, ax=ax2)
ax2.set_title('Height Map')
ax2.set_xlabel('x')
ax2.set_ylabel('y')

# Color Heatmap
ax3 = fig.add_subplot(133)
ax3.imshow(rgb_image, extent=[X_RANGE[0], X_RANGE[1],
           Y_RANGE[0], Y_RANGE[1]], origin='lower')
ax3.set_title('Heatmap')
ax3.set_xlabel('x')
ax3.set_ylabel('y')

plt.show()

print(f"Random seed: {random_seed}")
print(f"x0: {x0}, y0: {y0}")
print(f"sigma_x: {sigma_x}, sigma_y: {sigma_y}")
print(f"A: {A}")
print(f"Winner coordinate (grid index): {winner_coordinate}")
print(f"Continuous max point (x0, y0): ({x0}, {y0})")
