interface ImageCapture {
  grabFrame(): Promise<ImageBitmap>;
}

declare const ImageCapture: {
  new (track: MediaStreamTrack): ImageCapture;
};
